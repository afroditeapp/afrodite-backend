use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, AccountState, Capabilities, SharedState, SharedStateInternal, NextQueueNumbersRaw, NextQueueNumberType};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};
use tokio_stream::StreamExt;

use crate::IntoDatabaseError;

mod state;
mod queue_number;

define_read_commands!(CurrentReadAccount, CurrentSyncReadCommon);

impl<C: ConnectionProvider> CurrentSyncReadCommon<C> {
    pub fn state(self) -> state::CurrentSyncReadCommonState<C> {
        state::CurrentSyncReadCommonState::new(self.cmds)
    }

    pub fn queue_number(self) -> queue_number::CurrentSyncReadCommonQueueNumber<C> {
        queue_number::CurrentSyncReadCommonQueueNumber::new(self.cmds)
    }
}
