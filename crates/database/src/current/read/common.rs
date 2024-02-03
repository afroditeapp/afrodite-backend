use simple_backend_database::diesel_db::ConnectionProvider;

mod queue_number;
mod state;

define_read_commands!(CurrentReadAccount, CurrentSyncReadCommon);

impl<C: ConnectionProvider> CurrentSyncReadCommon<C> {
    pub fn state(self) -> state::CurrentSyncReadCommonState<C> {
        state::CurrentSyncReadCommonState::new(self.cmds)
    }

    pub fn queue_number(self) -> queue_number::CurrentSyncReadCommonQueueNumber<C> {
        queue_number::CurrentSyncReadCommonQueueNumber::new(self.cmds)
    }
}
