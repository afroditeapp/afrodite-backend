use database::{current::read::GetDbReadCommandsCommon, define_current_write_commands, DieselDatabaseError};
use diesel::{delete, insert_into, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{SyncVersion, UnixTime};
use model_media::{
    AccountIdInternal, ContentId, ContentIdDb, ContentModerationState, ContentSlot, MediaContentRaw, MediaContentType, NewContentParams, ProfileContentVersion, SetProfileContent
};
use simple_backend_utils::ContextExt;

use crate::{current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia}, IntoDatabaseError};

use super::DeletedSomething;

define_current_write_commands!(CurrentWriteMediaContent);

impl CurrentWriteMediaContent<'_> {
    pub fn insert_current_account_media(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let version = ProfileContentVersion::new_random();

        insert_into(current_account_media)
            .values((
                account_id.eq(id.as_db_id()),
                profile_content_version_uuid.eq(version),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Helper function for checking is ContentId valid to be set as profile
    /// or security content.
    ///
    /// Requirements:
    /// - The content must be in the `available` content list.
    /// - The content type must be JpegImage.
    /// - The provided check function must return true for the content.
    fn check_content_id(
        id: Option<ContentId>,
        available: &[MediaContentRaw],
        validate_state: impl Fn(&MediaContentRaw) -> bool,
    ) -> Result<Option<ContentIdDb>, DieselDatabaseError> {
        if let Some(content_id) = id {
            let found = available.iter().find(|content| {
                content.content_id() == content_id
                    && content.content_type() == MediaContentType::JpegImage
                    && validate_state(content)
            });

            if let Some(c) = found {
                Ok(Some(c.content_row_id()))
            } else {
                Err(DieselDatabaseError::NotAllowed.report())
            }
        } else {
            Ok(None)
        }
    }

    /// Update or delete current profile content if possible.
    ///
    /// Moves content to moderation if needed.
    ///
    /// Updates also [model_media::ProfileContentSyncVersion].
    ///
    /// Requirements:
    ///  - The content must be of type JpegImage.
    ///  - The content must be in the account's media content.
    ///  - The first content must have face detected flag set.
    pub fn update_profile_content(
        &mut self,
        id: AccountIdInternal,
        new: SetProfileContent,
        new_version: ProfileContentVersion,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(id)?;
        let convert_first = |content_id: Option<ContentId>| {
            Self::check_content_id(content_id, &all_content, |c| {
                c.face_detected
            })
        };
        let convert = |content_id: Option<ContentId>| {
            Self::check_content_id(content_id, &all_content, |_| {
                true
            })
        };

        update(current_account_media.find(id.as_db_id()))
            .set((
                profile_content_version_uuid.eq(new_version),
                profile_content_id_0.eq(convert_first(Some(new.c0))?),
                profile_content_id_1.eq(convert(new.c1)?),
                profile_content_id_2.eq(convert(new.c2)?),
                profile_content_id_3.eq(convert(new.c3)?),
                profile_content_id_4.eq(convert(new.c4)?),
                profile_content_id_5.eq(convert(new.c5)?),
                grid_crop_size.eq(new.grid_crop_size),
                grid_crop_x.eq(new.grid_crop_x),
                grid_crop_y.eq(new.grid_crop_y),
            ))
            .execute(self.conn())
            .into_db_error((id, new))?;

        for content_id in new.iter() {
            let state = self
                .read()
                .media()
                .media_content()
                .get_media_content_raw(content_id)?;
            if state.state().is_in_slot() {
                self
                    .write()
                    .media_admin()
                    .media_content()
                    .update_content_moderation_state(content_id, ContentModerationState::WaitingBotOrHumanModeration)?;
            }
        }

        Ok(())
    }

    /// Update security content if possible.
    ///
    /// Moves content to moderation if needed.
    ///
    /// Requirements:
    /// - The content must be of type JpegImage.
    /// - The content must be in the account's media content.
    /// - The content must have secure capture flag enabled.
    /// - The content must have face detected flag enabled.
    pub fn update_security_content(
        &mut self,
        content_owner: AccountIdInternal,
        content_id: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(content_owner)?;

        let content_db_id = Self::check_content_id(Some(content_id), &all_content, |c| {
            c.secure_capture && c.face_detected
        })?;

        update(current_account_media.find(content_owner.as_db_id()))
            .set((security_content_id.eq(content_db_id),))
            .execute(self.conn())
            .into_db_error((content_owner, content_id))?;

        let state = self
            .read()
            .media()
            .media_content()
            .get_media_content_raw(content_id)?;
        if state.state().is_in_slot() {
            self
                .write()
                .media_admin()
                .media_content()
                .update_content_moderation_state(content_id, ContentModerationState::WaitingBotOrHumanModeration)?;
        }

        Ok(())
    }

    /// Delete content from account's media content if possible
    ///
    /// Requirements:
    /// - The content must be in the account's media content.
    /// - The content must not be set in account's current or pending
    ///   media content.
    /// - The content must not be in moderation.
    pub fn delete_content(
        &mut self,
        content_owner: AccountIdInternal,
        content_id: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        let selected_content = self
            .read()
            .media()
            .media_content()
            .current_account_media(content_owner)?;
        let selected_content = selected_content
            .iter_all_content()
            .find(|c| c.content_id() == content_id);
        if selected_content.is_some() {
            return Err(DieselDatabaseError::ContentIsInUse.report());
        }

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(content_owner)?;
        let found_content = all_content.iter().find(|c| c.content_id() == content_id);

        if let Some(c) = found_content {
            // TODO(prod): Content not in use time tracking
            if c.state().is_in_moderation()  {
                Err(DieselDatabaseError::ContentIsInUse.report())
            } else {
                use model::schema::media_content::dsl::*;
                delete(media_content.filter(id.eq(c.content_row_id())))
                    .execute(self.conn())
                    .into_db_error((content_owner, content_id))?;
                Ok(())
            }
        } else {
            Err(DieselDatabaseError::NotAllowed.report())
        }
    }

    pub fn insert_content_id_to_slot(
        &mut self,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ContentSlot,
        content_params: NewContentParams,
        face_detected_value: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        let current_time = UnixTime::current_time();

        let account = self.read().common().account(content_uploader)?;
        let initial_content_value = account.profile_visibility().is_pending();

        insert_into(media_content)
            .values((
                account_id.eq(content_uploader.as_db_id()),
                uuid.eq(content_id),
                slot_number.eq(slot as i64),
                secure_capture.eq(content_params.secure_capture),
                face_detected.eq(face_detected_value),
                content_type_number.eq(content_params.content_type),
                initial_content.eq(initial_content_value),
                creation_unix_time.eq(current_time),
            ))
            .execute(self.conn())
            .into_db_error((content_uploader, content_id, slot))?;

        Ok(())
    }

    pub fn delete_content_from_slot(
        &mut self,
        request_creator: AccountIdInternal,
        slot: ContentSlot,
    ) -> Result<Option<DeletedSomething>, DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        let deleted_count = delete(
            media_content
                .filter(account_id.eq(request_creator.as_db_id()))
                .filter(moderation_state.eq(ContentModerationState::InSlot))
                .filter(slot_number.eq(slot as i64)),
        )
        .execute(self.conn())
        .into_db_error((request_creator, slot))?;

        if deleted_count > 0 {
            Ok(Some(DeletedSomething))
        } else {
            Ok(None)
        }
    }

    pub fn increment_profile_content_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_state::dsl::*;

        update(media_state)
            .filter(account_id.eq(id.as_db_id()))
            .filter(profile_content_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(profile_content_sync_version.eq(profile_content_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn only_profile_content_version(
        &mut self,
        id: AccountIdInternal,
        data: ProfileContentVersion,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::current_account_media::dsl::*;

        update(current_account_media.find(id.as_db_id()))
            .set(profile_content_version_uuid.eq(data))
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }
}
