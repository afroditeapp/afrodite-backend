use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*, upsert::excluded};
use error_stack::Result;
use model::AccountIdInternal;
use model_account::{CustomEmailId, UpdateCustomEmail};
use simple_backend_utils::db::MyRunQueryDsl;

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
                .select(sending_initiated)
                .first::<bool>(self.conn())
                .into_db_error(())?
            {
                return Err(error_stack::report!(DieselDatabaseError::NotAllowed));
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
}
