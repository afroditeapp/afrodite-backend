use database::{DieselDatabaseError, define_current_write_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::UnixTime;
use model_chat::AccountIdInternal;
use simple_backend_utils::time::DurationValue;

pub use crate::current::read::chat::transfer::TransferBudgetCheckResult;
use crate::{IntoDatabaseError, current::read::GetDbReadCommandsChat};

define_current_write_commands!(CurrentWriteChatTransfer);

impl CurrentWriteChatTransfer<'_> {
    /// Update the transfer budget with the actual bytes transferred.
    /// This should be called after the transfer is complete with the actual number of bytes.
    /// Resets the counter if a year has passed.
    pub fn update_transfer_budget(
        &mut self,
        id: AccountIdInternal,
        actual_bytes_transferred: i64,
        yearly_limit_bytes: i64,
    ) -> Result<TransferBudgetCheckResult, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        let state = self.read().chat().chat_state(id)?;

        let (current_count, reset_time) = match state.data_transfer_byte_count_reset_unix_time {
            None => (0, UnixTime::current_time()),
            Some(reset_time)
                if reset_time.duration_value_elapsed(DurationValue::from_days(365)) =>
            {
                (0, UnixTime::current_time())
            }
            Some(reset_time) => (state.data_transfer_byte_count, reset_time),
        };

        let new_total = current_count + actual_bytes_transferred;

        if new_total > yearly_limit_bytes {
            return Ok(TransferBudgetCheckResult::ExceedsLimit);
        }

        diesel::update(chat_state.find(id.as_db_id()))
            .set((
                data_transfer_byte_count.eq(new_total),
                data_transfer_byte_count_reset_unix_time.eq(reset_time),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(TransferBudgetCheckResult::Ok)
    }
}
