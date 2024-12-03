use database::{current::read::GetDbReadCommandsCommon, define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_media::{
    AccountIdInternal, ContentId, ContentIdDb, ContentModerationState, ContentSlot, CurrentAccountMediaInternal, CurrentAccountMediaRaw, MediaContentRaw
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadMediaMediaContent);

impl CurrentReadMediaMediaContent<'_> {
    fn media_content_raw(
        &mut self,
        media_owner_id: AccountIdInternal,
        id: Option<ContentIdDb>,
    ) -> Result<Option<MediaContentRaw>, DieselDatabaseError> {
        if let Some(content_id) = id {
            use crate::schema::media_content::dsl::*;

            let content = media_content
                .filter(id.eq(content_id))
                .select(MediaContentRaw::as_select())
                .first(self.conn())
                .into_db_error((media_owner_id, content_id))?;

            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    pub fn current_account_media_raw(
        &mut self,
        media_owner_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaRaw, DieselDatabaseError> {
        use crate::schema::current_account_media;

        current_account_media::table
            .filter(current_account_media::account_id.eq(media_owner_id.as_db_id()))
            .select(CurrentAccountMediaRaw::as_select())
            .first::<CurrentAccountMediaRaw>(self.conn())
            .into_db_error(media_owner_id)
    }

    pub fn current_account_media(
        &mut self,
        media_owner_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DieselDatabaseError> {
        let raw = self.current_account_media_raw(media_owner_id)?;

        let security_content_id =
            self.media_content_raw(media_owner_id, raw.security_content_id)?;
        let profile_content_id_0 =
            self.media_content_raw(media_owner_id, raw.profile_content_id_0)?;
        let profile_content_id_1 =
            self.media_content_raw(media_owner_id, raw.profile_content_id_1)?;
        let profile_content_id_2 =
            self.media_content_raw(media_owner_id, raw.profile_content_id_2)?;
        let profile_content_id_3 =
            self.media_content_raw(media_owner_id, raw.profile_content_id_3)?;
        let profile_content_id_4 =
            self.media_content_raw(media_owner_id, raw.profile_content_id_4)?;
        let profile_content_id_5 =
            self.media_content_raw(media_owner_id, raw.profile_content_id_5)?;

        Ok(CurrentAccountMediaInternal {
            grid_crop_size: raw.grid_crop_size,
            grid_crop_x: raw.grid_crop_x,
            grid_crop_y: raw.grid_crop_y,
            profile_content_version_uuid: raw.profile_content_version_uuid,
            security_content_id,
            profile_content_id_0,
            profile_content_id_1,
            profile_content_id_2,
            profile_content_id_3,
            profile_content_id_4,
            profile_content_id_5,
        })
    }

    pub fn get_media_content_raw(
        &mut self,
        content_id: ContentId,
    ) -> Result<MediaContentRaw, DieselDatabaseError> {
        use crate::schema::media_content::dsl::*;
        let content = media_content
            .filter(uuid.eq(content_id))
            .select(MediaContentRaw::as_select())
            .first(self.conn())
            .into_db_error(content_id)?;
        Ok(content)
    }

    pub fn get_media_content_raw_with_account_id(
        &mut self,
        content_id: ContentId,
    ) -> Result<(MediaContentRaw, AccountIdInternal), DieselDatabaseError> {
        let content = self.get_media_content_raw(content_id)?;
        let account_id_value = self.read().common().db_id_to_internal_id(content.account_id)?;
        Ok((content, account_id_value))
    }

    pub fn get_account_media_content(
        &mut self,
        media_owner_id: AccountIdInternal,
    ) -> Result<Vec<MediaContentRaw>, DieselDatabaseError> {
        use crate::schema::media_content::dsl::*;

        media_content
            .filter(account_id.eq(media_owner_id.as_db_id()))
            .select(MediaContentRaw::as_select())
            .load(self.conn())
            .into_db_error(media_owner_id)
    }

    pub fn get_media_content_from_slot(
        &mut self,
        slot_owner: AccountIdInternal,
        slot: ContentSlot,
    ) -> Result<Option<MediaContentRaw>, DieselDatabaseError> {
        use crate::schema::media_content::dsl::*;

        media_content
            .filter(account_id.eq(slot_owner.as_db_id()))
            .filter(moderation_state.eq(ContentModerationState::InSlot))
            .filter(slot_number.eq(slot))
            .select(MediaContentRaw::as_select())
            .first(self.conn())
            .optional()
            .into_db_error((slot_owner, slot))
    }
}
