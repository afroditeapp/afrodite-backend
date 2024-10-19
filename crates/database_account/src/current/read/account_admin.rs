use database::{define_current_read_commands, ConnectionProvider};

define_current_read_commands!(CurrentReadAccountAdmin, CurrentSyncReadAccountAdmin);

mod news;

impl<C: ConnectionProvider> CurrentSyncReadAccountAdmin<C> {
    pub fn news(self) -> news::CurrentSyncReadAccountNewsAdmin<C> {
        news::CurrentSyncReadAccountNewsAdmin::new(self.cmds)
    }
}
