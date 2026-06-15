use database::{DieselDatabaseError, define_current_read_commands};
use diesel::{ExpressionMethods, prelude::*};
use error_stack::{Result, ResultExt};
use model::AccountIdInternal;
use model_account::{
    CustomEmail, CustomEmailId, CustomEmailInternal, CustomEmailTranslation,
    CustomEmailTranslationInternal,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountCustomEmailAdmin);

impl CurrentReadAccountCustomEmailAdmin<'_> {
    /// Returns all custom emails ordered by id DESC (newest first).
    pub fn custom_email_list_page(
        &mut self,
        page_number: i64,
    ) -> Result<Vec<CustomEmail>, DieselDatabaseError> {
        use crate::schema::custom_email;

        const PAGE_SIZE: i64 = 25;

        let offsets = page_number * PAGE_SIZE;

        let email_items: Vec<CustomEmailInternal> = custom_email::table
            .order(custom_email::id.desc())
            .limit(PAGE_SIZE)
            .offset(offsets)
            .select(CustomEmailInternal::as_select())
            .load(self.conn())
            .into_db_error(())?;

        let mut result = Vec::with_capacity(email_items.len());
        for item in email_items {
            let translations: Vec<CustomEmailTranslation> = {
                use crate::schema::custom_email_translations::dsl::*;
                custom_email_translations
                    .filter(email_id.eq(item.id))
                    .select(CustomEmailTranslationInternal::as_select())
                    .load(self.conn())
                    .into_db_error(())?
                    .into_iter()
                    .map(|internal| CustomEmailTranslation {
                        subject: internal.message_subject,
                        body: internal.message_body,
                        locale: internal.locale,
                    })
                    .collect()
            };

            result.push(CustomEmail {
                id: item.id,
                sending_initiated_unix_time: item.sending_initiated_unix_time,
                sending_completed_unix_time: item.sending_completed_unix_time,
                translations,
            });
        }

        Ok(result)
    }

    pub fn custom_emails_pending_sending(
        &mut self,
    ) -> Result<Vec<CustomEmailId>, DieselDatabaseError> {
        use crate::schema::custom_email::dsl::*;

        let ids: Vec<CustomEmailId> = custom_email
            .filter(sending_initiated_unix_time.is_not_null())
            .filter(sending_completed_unix_time.is_null())
            .select(id)
            .load(self.conn())
            .into_db_error(())?;

        Ok(ids)
    }

    pub fn custom_email_unsent_accounts(
        &mut self,
        email_id_value: CustomEmailId,
    ) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::custom_email_sending_state::dsl::*;

        let accounts: Vec<AccountIdInternal> = custom_email_sending_state
            .filter(email_id.eq(email_id_value))
            .filter(email_sent.eq(false))
            .inner_join(crate::schema::account_id::table)
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .into_db_error(())?;

        Ok(accounts)
    }

    pub fn custom_email_translations(
        &mut self,
        email_id_value: CustomEmailId,
    ) -> Result<Vec<CustomEmailTranslation>, DieselDatabaseError> {
        use crate::schema::custom_email_translations::dsl::*;

        let rows: Vec<CustomEmailTranslationInternal> = custom_email_translations
            .filter(email_id.eq(email_id_value))
            .select(CustomEmailTranslationInternal::as_select())
            .load(self.conn())
            .into_db_error(())?;

        let result = rows
            .into_iter()
            .map(|r| CustomEmailTranslation {
                subject: r.message_subject,
                body: r.message_body,
                locale: r.locale,
            })
            .collect();

        Ok(result)
    }

    pub fn custom_email_sending_limits(
        &mut self,
    ) -> Result<Option<model::CustomEmailSendingLimits>, DieselDatabaseError> {
        use crate::schema::custom_email_sending_limits::dsl::*;

        custom_email_sending_limits
            .filter(row_type.eq(0))
            .select(model::CustomEmailSendingLimits::as_select())
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }
}
