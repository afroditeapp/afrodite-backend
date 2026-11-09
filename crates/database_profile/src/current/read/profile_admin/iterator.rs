use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::ResultExt;
use model::{
    AccountIdDb, AccountIdInternal, AccountState, AccountStateRelatedSharedState,
    LoginSessionInfoInternal,
};
use model_profile::{
    AccountIdDbValue, ProfileInternal, ProfileIteratorPageData, ProfileIteratorSettings,
    ProfileStringIterationPage, ProfileStringIterationValue, ProfileStringModerationContentType,
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

    /// Also returns if there are more pages after the requested page.
    pub fn get_profile_page(
        &mut self,
        settings: ProfileIteratorSettings,
    ) -> Result<(Vec<ProfileIteratorPageData>, bool), DieselDatabaseError> {
        use crate::schema::{account_id, profile, shared_state};

        const PAGE_SIZE: i64 = 25;

        let filters = settings.filters();

        let mut query = account_id::table
            .inner_join(profile::table)
            .inner_join(shared_state::table)
            .select((
                AccountIdInternal::as_select(),
                ProfileInternal::as_select(),
                AccountStateRelatedSharedState::as_select(),
            ))
            .into_boxed();

        query = query
            .filter(account_id::id.le(settings.start_position))
            .order(account_id::id.desc());

        if let Some(account_state) = filters.account_state {
            match account_state {
                AccountState::InitialSetup => {
                    query = query
                        .filter(shared_state::account_state_banned.eq(false))
                        .filter(shared_state::account_state_pending_deletion.eq(false))
                        .filter(shared_state::account_state_initial_setup_completed.eq(false));
                }
                AccountState::Normal => {
                    query = query
                        .filter(shared_state::account_state_banned.eq(false))
                        .filter(shared_state::account_state_pending_deletion.eq(false))
                        .filter(shared_state::account_state_initial_setup_completed.eq(true));
                }
                AccountState::Banned => {
                    query = query
                        .filter(shared_state::account_state_banned.eq(true))
                        .filter(shared_state::account_state_pending_deletion.eq(false));
                }
                AccountState::PendingDeletion => {
                    query = query.filter(shared_state::account_state_pending_deletion.eq(true));
                }
            }
        }

        if let Some(visibility) = filters.profile_visibility {
            query = query.filter(shared_state::profile_visibility_state_number.eq(visibility));
        }

        // Fetch one extra row to detect if there are more pages after
        // the requested page.
        let limit = PAGE_SIZE.saturating_add(1);

        let data: Vec<(
            AccountIdInternal,
            ProfileInternal,
            AccountStateRelatedSharedState,
        )> = query
            .limit(limit)
            .offset(PAGE_SIZE.saturating_mul(settings.page))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        let has_more_pages = data.len() > PAGE_SIZE.try_into().unwrap_or(usize::MAX);

        Ok((
            data.into_iter()
                .take(PAGE_SIZE.try_into().unwrap_or(usize::MAX))
                .map(|(id, profile, state)| ProfileIteratorPageData { id, profile, state })
                .collect(),
            has_more_pages,
        ))
    }

    pub fn get_email_addresses(
        &mut self,
        account_db_ids: &[AccountIdDb],
    ) -> Result<Vec<(AccountIdDb, Option<String>)>, DieselDatabaseError> {
        use crate::schema::account_email_address_state;

        if account_db_ids.is_empty() {
            return Ok(vec![]);
        }

        account_email_address_state::table
            .filter(account_email_address_state::account_id.eq_any(account_db_ids.to_vec()))
            .select((
                account_email_address_state::account_id,
                account_email_address_state::email.nullable(),
            ))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn get_login_session_infos(
        &mut self,
        account_db_ids: &[AccountIdDb],
    ) -> Result<Vec<(AccountIdDb, LoginSessionInfoInternal)>, DieselDatabaseError> {
        use crate::schema::login_session_info;

        if account_db_ids.is_empty() {
            return Ok(vec![]);
        }

        login_session_info::table
            .filter(login_session_info::account_id.eq_any(account_db_ids.to_vec()))
            .select((
                login_session_info::account_id,
                LoginSessionInfoInternal::as_select(),
            ))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)
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
