


use simple_backend_database::diesel_db::{ConnectionProvider};




mod interaction;
mod message;

define_write_commands!(CurrentWriteChat, CurrentSyncWriteChat);

impl<C: ConnectionProvider> CurrentSyncWriteChat<C> {
    pub fn interaction(self) -> interaction::CurrentSyncWriteChatInteraction<C> {
        interaction::CurrentSyncWriteChatInteraction::new(self.cmds)
    }

    pub fn message(self) -> message::CurrentSyncWriteChatMessage<C> {
        message::CurrentSyncWriteChatMessage::new(self.cmds)
    }
}
