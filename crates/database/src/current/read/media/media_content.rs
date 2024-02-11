use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountIdInternal, ContentId, ContentIdDb, CurrentAccountMediaInternal, CurrentAccountMediaRaw,
    MediaContentRaw,
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(
    CurrentReadMediaMediaContent,
    CurrentSyncReadMediaMediaContent
);

impl<C: ConnectionProvider> CurrentSyncReadMediaMediaContent<C> {
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

    pub fn current_account_media(
        &mut self,
        media_owner_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DieselDatabaseError> {
        use crate::schema::current_account_media;

        let raw = current_account_media::table
            .filter(current_account_media::account_id.eq(media_owner_id.as_db_id()))
            .select(CurrentAccountMediaRaw::as_select())
            .first::<CurrentAccountMediaRaw>(self.conn())
            .into_db_error(media_owner_id)?;

        let security_content_id =
            self.media_content_raw(media_owner_id, raw.security_content_id)?;
        let pending_security_content_id =
            self.media_content_raw(media_owner_id, raw.pending_security_content_id)?;
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
        let pending_profile_content_id_0 =
            self.media_content_raw(media_owner_id, raw.pending_profile_content_id_0)?;
        let pending_profile_content_id_1 =
            self.media_content_raw(media_owner_id, raw.pending_profile_content_id_1)?;
        let pending_profile_content_id_2 =
            self.media_content_raw(media_owner_id, raw.pending_profile_content_id_2)?;
        let pending_profile_content_id_3 =
            self.media_content_raw(media_owner_id, raw.pending_profile_content_id_3)?;
        let pending_profile_content_id_4 =
            self.media_content_raw(media_owner_id, raw.pending_profile_content_id_4)?;
        let pending_profile_content_id_5 =
            self.media_content_raw(media_owner_id, raw.pending_profile_content_id_5)?;

        Ok(CurrentAccountMediaInternal {
            grid_crop_size: raw.grid_crop_size,
            grid_crop_x: raw.grid_crop_x,
            grid_crop_y: raw.grid_crop_y,
            pending_grid_crop_size: raw.pending_grid_crop_size,
            pending_grid_crop_x: raw.pending_grid_crop_x,
            pending_grid_crop_y: raw.pending_grid_crop_y,
            security_content_id,
            pending_security_content_id,
            profile_content_id_0,
            profile_content_id_1,
            profile_content_id_2,
            profile_content_id_3,
            profile_content_id_4,
            profile_content_id_5,
            pending_profile_content_id_0,
            pending_profile_content_id_1,
            pending_profile_content_id_2,
            pending_profile_content_id_3,
            pending_profile_content_id_4,
            pending_profile_content_id_5,
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
}
