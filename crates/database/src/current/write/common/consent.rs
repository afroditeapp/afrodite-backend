use diesel::{insert_into, prelude::*};
use model::{AccountIdInternal, UnixTime, UpdateDataConsentSettings};
use simple_backend_utils::{Result, db::MyRunQueryDsl};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonConsent);

impl CurrentWriteCommonConsent<'_> {
    pub fn upsert_data_consent_settings(
        &mut self,
        id: AccountIdInternal,
        value: UpdateDataConsentSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::data_consent_settings::dsl::*;

        let account_db_id = id.as_db_id();
        let time = UnixTime::current_time();

        if let Some(allow_face_verification_value) = value.allow_face_verification {
            insert_into(data_consent_settings)
                .values((
                    account_id.eq(account_db_id),
                    allow_face_verification.eq(allow_face_verification_value),
                    allow_face_verification_edit_unix_time.eq(time),
                ))
                .on_conflict(account_id)
                .do_update()
                .set((
                    allow_face_verification.eq(allow_face_verification_value),
                    allow_face_verification_edit_unix_time.eq(time),
                ))
                .execute_my_conn(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }
}
