use diesel::{insert_into, prelude::*, update, ExpressionMethods, QueryDsl};
use error_stack::Result;
use model::{
    AccountIdInternal, LocationIndexKey, ProfileInternal, ProfileUpdateInternal, ProfileVersion, Profile,
};
use utils::{IntoReportExt, current_unix_time};

use crate::{diesel::{DieselDatabaseError, HistoryConnectionProvider}, IntoDatabaseError};

define_write_commands!(HistoryWriteProfile, HistorySyncWriteProfile);

impl<C: HistoryConnectionProvider> HistorySyncWriteProfile<C> {

    pub fn insert_profile(
        &mut self,
        account_id_internal: AccountIdInternal,
        profile: &Profile,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::history_profile::dsl::*;

        let text = serde_json::to_string(profile)
            .into_error(DieselDatabaseError::SerdeSerialize)?;
        let time = current_unix_time();

        insert_into(history_profile)
            .values((
                account_id.eq(account_id_internal.as_db_id()),
                unix_time.eq(time),
                json_text.eq(text),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (account_id_internal, account_id_internal))?;
        Ok(())
    }
}
