use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::AccountIdInternal;
use model_profile::{
    ProfileModerationContentType, ProfileModerationInfo, ProfileNameModerationState,
    ProfileTextModerationState,
};

define_current_read_commands!(CurrentReadProfileModeration);

impl CurrentReadProfileModeration<'_> {
    pub fn profile_name_moderation_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<ProfileNameModerationState>, DieselDatabaseError> {
        use crate::schema::profile_moderation::dsl::*;

        profile_moderation
            .filter(account_id.eq(id.as_db_id()))
            .filter(content_type.eq(ProfileModerationContentType::ProfileName))
            .select(state_type)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn profile_text_moderation_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<ProfileTextModerationState>, DieselDatabaseError> {
        use crate::schema::profile_moderation::dsl::*;

        profile_moderation
            .filter(account_id.eq(id.as_db_id()))
            .filter(content_type.eq(ProfileModerationContentType::ProfileText))
            .select(state_type)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn profile_moderation_info(
        &mut self,
        id: AccountIdInternal,
        moderation_info: ProfileModerationContentType,
    ) -> Result<Option<ProfileModerationInfo>, DieselDatabaseError> {
        use crate::schema::profile_moderation::dsl::*;

        profile_moderation
            .filter(account_id.eq(id.as_db_id()))
            .filter(content_type.eq(moderation_info))
            .select(ProfileModerationInfo::as_select())
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn is_name_on_database_allowlist(
        &mut self,
        name: &str,
    ) -> Result<bool, DieselDatabaseError> {
        use crate::schema::profile_name_allowlist::dsl::*;

        let exists = profile_name_allowlist
            .filter(profile_name.eq(&name))
            .select(name_creator_account_id)
            .first::<i64>(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        Ok(exists.is_some())
    }
}
