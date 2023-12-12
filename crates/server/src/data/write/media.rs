use database::{current::write::media::CurrentSyncWriteMedia};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ContentId, ImageSlot, ModerationRequestContent, PrimaryImage};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::data::DataError;

define_write_commands!(WriteCommandsMedia);

impl WriteCommandsMedia<'_> {
    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DataError> {
        self.db_write(move |mut cmds| {
            cmds.media()
                .create_new_moderation_request(account_id, request)
        })
        .await
    }

    /// Completes previous save_to_tmp.
    pub async fn save_to_slot(
        &self,
        id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DataError> {
        // Remove previous slot image.
        let current_content_in_slot = self
            .db_read(move |mut cmds| cmds.media().get_content_id_from_slot(id, slot))
            .await?;

        if let Some(current_id) = current_content_in_slot {
            let path = self
                .file_dir()
                .image_content(id.as_id(), current_id.as_content_id());
            path.remove_if_exists()
                .await
                .change_context(DataError::File)?;
            self.db_write(move |mut cmds| cmds.media().delete_image_from_slot(id, slot))
                .await
                .change_context(DataError::Sqlite)?;
        }

        // Paths related to moving image from tmp to image dir
        let tmp_img = self
            .file_dir()
            .processed_image_upload(id.as_id(), content_id);
        let processed_content_path = self.file_dir().image_content(id.as_id(), content_id);

        if self
            .db_read(move |mut cmds| cmds.media().get_content_id_from_slot(id, slot))
            .await?
            .is_some()
        {
            return Err(DataError::ContentSlotNotEmpty.report());
        }

        self.db_transaction(move |conn| {
            CurrentSyncWriteMedia::insert_content_id_to_slot(conn, id, content_id, slot)?;

            // Move image from tmp to image dir
            tmp_img
                .move_to_blocking(&processed_content_path)
                .change_context(DieselDatabaseError::File)?;
            // If moving fails, diesel rollbacks the transaction.

            Ok(())
        })
        .await?;

        // TODO: Update media backup code
        // self.media_backup()
        //     .backup_jpeg_image(id.as_id(), content_id)
        //     .await
        //     .change_context(DataError::MediaBackup)

        Ok(())
    }

    pub async fn update_primary_image(
        self,
        id: AccountIdInternal,
        primary_image: PrimaryImage,
    ) -> Result<(), DataError> {
        self.db_write(move |mut cmds| cmds.media().primary_image(id, primary_image))
            .await
    }
}
