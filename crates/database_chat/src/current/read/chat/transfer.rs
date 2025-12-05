use database::{DieselDatabaseError, define_current_read_commands};
use error_stack::Result;
use model_chat::AccountIdInternal;
use simple_backend_utils::time::DurationValue;

use crate::current::read::GetDbReadCommandsChat;

define_current_read_commands!(CurrentReadChatTransfer);

pub enum TransferBudgetCheckResult {
    /// Transfer is allowed, returns the current budget used
    Ok,
    /// Transfer exceeds budget limit
    ExceedsLimit,
}

impl CurrentReadChatTransfer<'_> {
    /// Check if a transfer of the given size is allowed within the yearly budget.
    pub fn check_transfer_budget(
        &mut self,
        id: AccountIdInternal,
        transfer_bytes: u32,
        yearly_limit_bytes: i64,
    ) -> Result<TransferBudgetCheckResult, DieselDatabaseError> {
        let state = self.read().chat().chat_state(id)?;

        let current_count = match state.data_transfer_byte_count_reset_unix_time {
            None => 0,
            Some(reset_time)
                if reset_time.duration_value_elapsed(DurationValue::from_days(365)) =>
            {
                0
            }
            Some(_) => state.data_transfer_byte_count,
        };

        if current_count + Into::<i64>::into(transfer_bytes) > yearly_limit_bytes {
            Ok(TransferBudgetCheckResult::ExceedsLimit)
        } else {
            Ok(TransferBudgetCheckResult::Ok)
        }
    }
}
