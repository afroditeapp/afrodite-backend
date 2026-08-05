use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*, update, upsert::excluded};
use error_stack::IntoReport;
use model::{AccountIdInternal, CustomEmailSendingLimits, UnixTime};
use model_account::{CustomEmailId, UpdateCustomEmail};
use simple_backend_utils::{Result, db::MyRunQueryDsl};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountCustomEmailAdmin);

impl CurrentWriteAccountCustomEmailAdmin<'_> {
    pub fn create_custom_email(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<CustomEmailId, DieselDatabaseError> {
        use crate::schema::custom_email::dsl::*;

        let email_id_value: CustomEmailId = insert_into(custom_email)
            .values(account_id_creator.eq(id_value.as_db_id()))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(email_id_value)
    }

    pub fn update_custom_email(
        &mut self,
        data: UpdateCustomEmail,
    ) -> Result<(), DieselDatabaseError> {
        // Check that sending is not initiated
        {
            use crate::schema::custom_email::dsl::*;
            if custom_email
                .filter(id.eq(data.id))
                .select(sending_initiated_unix_time)
                .first::<Option<UnixTime>>(self.conn())
                .into_db_error(())?
                .is_some()
            {
                return Err(DieselDatabaseError::NotAllowed.into_report());
            }
        }

        use crate::schema::custom_email_translations::dsl::*;

        // Delete all existing translations before re-inserting
        diesel::delete(custom_email_translations.filter(email_id.eq(data.id)))
            .execute(self.conn())
            .into_db_error(())?;

        for translation in &data.translations {
            insert_into(custom_email_translations)
                .values((
                    locale.eq(&translation.locale),
                    email_id.eq(data.id),
                    message_subject.eq(&translation.subject),
                    message_body.eq(&translation.body),
                ))
                .on_conflict((email_id, locale))
                .do_update()
                .set((
                    message_subject.eq(excluded(message_subject)),
                    message_body.eq(excluded(message_body)),
                ))
                .execute_my_conn(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

    pub fn init_custom_email_sending(
        &mut self,
        email_id_value: CustomEmailId,
        account_ids: &[AccountIdInternal],
    ) -> Result<(), DieselDatabaseError> {
        {
            use crate::schema::custom_email_sending_state::dsl::*;

            let rows: Vec<_> = account_ids
                .iter()
                .map(|id| {
                    (
                        email_id.eq(email_id_value),
                        account_id.eq(id.as_db_id()),
                        email_sent.eq(false),
                    )
                })
                .collect();

            insert_into(custom_email_sending_state)
                .values(&rows)
                .execute_my_conn(self.conn())
                .into_db_error(())?;
        }

        {
            use crate::schema::custom_email::dsl::*;

            update(custom_email)
                .filter(id.eq(email_id_value))
                .set(sending_initiated_unix_time.eq(UnixTime::current_time()))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

    pub fn mark_custom_email_sent(
        &mut self,
        email_id_value: CustomEmailId,
        account_id_value: &AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::custom_email_sending_state::dsl::*;

        update(custom_email_sending_state)
            .filter(email_id.eq(email_id_value))
            .filter(account_id.eq(account_id_value.as_db_id()))
            .set(email_sent.eq(true))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn set_custom_email_sending_completed(
        &mut self,
        email_id_value: CustomEmailId,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::custom_email::dsl::*;

        update(custom_email)
            .filter(id.eq(email_id_value))
            .set(sending_completed_unix_time.eq(UnixTime::current_time()))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_custom_email_sending_limits(
        &mut self,
        limits: &CustomEmailSendingLimits,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::custom_email_sending_limits::dsl::*;

        insert_into(custom_email_sending_limits)
            .values((
                row_type.eq(0),
                send_to_all_accounts_monthly_count.eq(limits.send_to_all_accounts_monthly_count),
                send_draft_to_my_email_monthly_count
                    .eq(limits.send_draft_to_my_email_monthly_count),
                reset_unix_time.eq(limits.reset_unix_time),
            ))
            .on_conflict(row_type)
            .do_update()
            .set((
                send_to_all_accounts_monthly_count.eq(limits.send_to_all_accounts_monthly_count),
                send_draft_to_my_email_monthly_count
                    .eq(limits.send_draft_to_my_email_monthly_count),
                reset_unix_time.eq(limits.reset_unix_time),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
