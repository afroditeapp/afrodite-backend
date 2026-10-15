use diesel::{insert_into, prelude::*};
use model::{AccountIdInternal, DataConsentSettings};
use simple_backend_utils::{Result, db::MyRunQueryDsl};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonConsent);

impl CurrentWriteCommonConsent<'_> {
    pub fn upsert_data_consent_settings(
        &mut self,
        id: AccountIdInternal,
        settings: DataConsentSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::data_consent_settings::dsl::*;

        insert_into(data_consent_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
