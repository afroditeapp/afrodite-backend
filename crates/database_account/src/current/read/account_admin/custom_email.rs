use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model_account::{
    CustomEmail, CustomEmailInternal, CustomEmailTranslation, CustomEmailTranslationInternal,
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
                sending_initiated: item.sending_initiated,
                sending_initiated_unix_time: item.sending_initiated_unix_time,
                sending_completed_unix_time: item.sending_completed_unix_time,
                translations,
            });
        }

        Ok(result)
    }
}
