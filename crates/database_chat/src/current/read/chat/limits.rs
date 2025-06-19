use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model_chat::{AccountIdInternal, DailyLikesLeftInternal};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatLimits);

impl CurrentReadChatLimits<'_> {
    pub fn daily_likes_left(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<DailyLikesLeftInternal, DieselDatabaseError> {
        use crate::schema::daily_likes_left::dsl::*;

        let query_result = daily_likes_left
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(DailyLikesLeftInternal::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }
}
