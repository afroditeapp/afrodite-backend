use model::{AccountIdInternal, ContentId, ImageSlot, ModerationRequestContent, PrimaryImage};

use crate::{data::DatabaseError, utils::ConvertCommandErrorExt};

use error_stack::{Report, Result, ResultExt};

define_write_commands!(WriteCommandsMedia);

impl WriteCommandsMedia<'_> {
    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DatabaseError> {
        self.current()
            .media()
            .create_new_moderation_request(account_id, request)
            .await
            .convert(account_id)
    }

    /// Completes previous save_to_tmp.
    pub async fn save_to_slot(
        &self,
        id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DatabaseError> {
        // Remove previous slot image.
        let current_content_in_slot = self
            .current_write()
            .read()
            .media()
            .get_content_id_from_slot(id, slot)
            .await
            .change_context(DatabaseError::Sqlite)?;
        if let Some(current_id) = current_content_in_slot {
            let path = self
                .file_dir()
                .image_content(id.as_light(), current_id.as_content_id());
            path.remove_if_exists()
                .await
                .change_context(DatabaseError::File)?;
            self.current()
                .media()
                .delete_image_from_slot(id, slot)
                .await
                .change_context(DatabaseError::Sqlite)?;
        }

        let cmds_current = self.current();
        let cmds_media = cmds_current.media();
        let transaction = cmds_media
            .store_content_id_to_slot(id, content_id, slot)
            .await
            .change_context(DatabaseError::Sqlite)?;

        let file_operations = || {
            async {
                // Move image from tmp to image dir
                let raw_img = self
                    .file_dir()
                    .unprocessed_image_upload(id.as_light(), content_id);
                let processed_content_path =
                    self.file_dir().image_content(id.as_light(), content_id);
                raw_img
                    .move_to(&processed_content_path)
                    .await
                    .change_context(DatabaseError::File)?;

                Ok::<(), Report<DatabaseError>>(())
            }
        };

        match file_operations().await {
            Ok(()) => {
                transaction
                    .commit()
                    .await
                    .change_context(DatabaseError::Sqlite)?;

                self.media_backup()
                    .backup_jpeg_image(id.as_light(), content_id)
                    .await
                    .change_context(DatabaseError::MediaBackup)?;

                Ok(())
            }
            Err(e) => {
                match transaction
                    .rollback()
                    .await
                    .change_context(DatabaseError::Sqlite)
                {
                    Ok(()) => Err(e),
                    Err(another_error) => Err(another_error.attach(e)),
                }
            }
        }
    }

    pub async fn update_primary_image(
        self,
        id: AccountIdInternal,
        primary_image: PrimaryImage,
    ) -> Result<(), DatabaseError> {
        self.current()
            .media()
            .update_current_account_media_with_primary_image(id, primary_image)
            .await
            .convert(id)
    }
}
