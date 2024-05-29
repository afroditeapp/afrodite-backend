use super::ConnectionProvider;

mod queue_number;
mod state;
mod token;

define_write_commands!(CurrentWriteAccount, CurrentSyncWriteCommon);

impl<C: ConnectionProvider> CurrentSyncWriteCommon<C> {
    pub fn queue_number(self) -> queue_number::CurrentSyncWriteCommonQueueNumber<C> {
        queue_number::CurrentSyncWriteCommonQueueNumber::new(self.cmds)
    }

    pub fn state(self) -> state::CurrentSyncWriteCommonState<C> {
        state::CurrentSyncWriteCommonState::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentSyncWriteAccountToken<C> {
        token::CurrentSyncWriteAccountToken::new(self.cmds)
    }
}
