use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model_chat::{AccountIdInternal, ChatPrivacySettings};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatPrivacy);

impl CurrentReadChatPrivacy<'_> {
    pub fn privacy_settings(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<ChatPrivacySettings, DieselDatabaseError> {
        use crate::schema::chat_privacy_settings::dsl::*;

        let query_result = chat_privacy_settings
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(ChatPrivacySettings::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }
}
