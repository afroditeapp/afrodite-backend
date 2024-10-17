use database::{define_current_write_commands, ConnectionProvider};

mod news;

define_current_write_commands!(CurrentWriteAccountAdmin, CurrentSyncWriteAccountAdmin);


impl<C: ConnectionProvider> CurrentSyncWriteAccountAdmin<C> {
    pub fn news(self) -> news::CurrentSyncWriteAccountNewsAdmin<C> {
        news::CurrentSyncWriteAccountNewsAdmin::new(self.cmds)
    }
}
