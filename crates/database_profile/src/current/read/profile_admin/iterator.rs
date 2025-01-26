use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdDb, AccountIdInternal};
use model_profile::{AccountIdDbValue, ProfileInternal, ProfileIteratorPage, ProfileIteratorPageValue, ProfileIteratorSettings};

define_current_read_commands!(CurrentReadProfileIterator);

impl CurrentReadProfileIterator<'_> {
    pub fn get_latest_account_id_db(
        &mut self,
    ) -> Result<AccountIdDbValue, DieselDatabaseError> {
        use crate::schema::account_id;

        let account_db_id: AccountIdDb = account_id::table
            .order((
                account_id::id.desc(),
            ))
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
            .order((
                account_id::id.desc(),
            ))
            .select((
                AccountIdInternal::as_select(),
                ProfileInternal::as_select(),
            ))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(settings.page))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        let values = data.into_iter()
            .map(|(id, profile)| ProfileIteratorPageValue {
                account_id: id.uuid,
                age: profile.age,
                name: profile.name,
            })
            .collect();

        Ok(ProfileIteratorPage { values })
    }
}
