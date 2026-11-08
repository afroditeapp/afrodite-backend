use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::ResultExt;
use model::{AccountIdDb, AccountIdInternal};
use model_profile::{
    AccountIdDbValue, ProfileInternal, ProfileIteratorPage, ProfileIteratorPageValue,
    ProfileIteratorSettings, ProfileStringIterationPage, ProfileStringIterationValue,
    ProfileStringModerationContentType,
};
use simple_backend_model::UnixTime;
use simple_backend_utils::Result;

define_current_read_commands!(CurrentReadProfileIterator);

impl CurrentReadProfileIterator<'_> {
    pub fn get_latest_account_id_db(&mut self) -> Result<AccountIdDbValue, DieselDatabaseError> {
        use crate::schema::account_id;

        let account_db_id: AccountIdDb = account_id::table
            .order((account_id::id.desc(),))
            .select(account_id::id)
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(AccountIdDbValue { account_db_id })
    }

    pub fn get_profile_page(
        &mut self,
        settings: ProfileIteratorSettings,
    ) -> Result<ProfileIteratorPage, DieselDatabaseError> {
        use crate::schema::{account_id, profile};

        const PAGE_SIZE: i64 = 25;

        let data: Vec<(AccountIdInternal, ProfileInternal)> = account_id::table
            .inner_join(profile::table)
            .filter(account_id::id.le(settings.start_position))
            .order((account_id::id.desc(),))
            .select((AccountIdInternal::as_select(), ProfileInternal::as_select()))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(settings.page))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        let values = data
            .into_iter()
            .map(|(id, profile)| ProfileIteratorPageValue {
                account_id: id.uuid,
                age: profile.age,
                name: profile.profile_name,
            })
            .collect();

        Ok(ProfileIteratorPage { values })
    }

    pub fn profile_string_iteration_page(
        &mut self,
        content_type: ProfileStringModerationContentType,
        page: i64,
        origin: UnixTime,
    ) -> Result<ProfileStringIterationPage, DieselDatabaseError> {
        use crate::schema::{account_id, profile, profile_state};

        const PAGE_SIZE: i64 = 25;

        let query = profile::table
            .inner_join(account_id::table)
            .inner_join(profile_state::table.on(profile_state::account_id.eq(account_id::id)))
            .into_boxed();

        let query = match content_type {
            ProfileStringModerationContentType::ProfileName => {
                query.filter(profile::profile_name.is_not_null()).select((
                    account_id::uuid,
                    profile::profile_name.assume_not_null(),
                    profile_state::profile_edited_unix_time,
                ))
            }
            ProfileStringModerationContentType::ProfileText => {
                query.filter(profile::profile_text.is_not_null()).select((
                    account_id::uuid,
                    profile::profile_text.assume_not_null(),
                    profile_state::profile_edited_unix_time,
                ))
            }
        };

        let query = query
            .filter(profile_state::profile_edited_unix_time.lt(origin.ut))
            .order((
                profile_state::profile_edited_unix_time.desc(),
                account_id::id.desc(),
            ));

        let values: Vec<ProfileStringIterationValue> = query
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(ProfileStringIterationPage { values })
    }
}
