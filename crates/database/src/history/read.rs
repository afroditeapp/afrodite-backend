use simple_backend_database::diesel_db::ConnectionProvider;

use self::{
    account::HistorySyncReadAccount, account_admin::HistorySyncReadAccountAdmin,
    chat::HistorySyncReadChat, chat_admin::HistorySyncReadChatAdmin, media::HistorySyncReadMedia,
    media_admin::HistorySyncReadMediaAdmin, profile::HistorySyncReadProfile,
    profile_admin::HistorySyncReadProfileAdmin,
};
use crate::HistoryReadHandle;

macro_rules! define_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

        pub struct $sync_name<C: simple_backend_database::diesel_db::ConnectionProvider> {
            cmds: C,
        }

        impl<C: simple_backend_database::diesel_db::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut simple_backend_database::diesel_db::DieselConnection {
                self.cmds.conn()
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> HistorySyncReadAccount<C> {
        HistorySyncReadAccount::new(self.conn)
    }

    pub fn into_account_admin(self) -> HistorySyncReadAccountAdmin<C> {
        HistorySyncReadAccountAdmin::new(self.conn)
    }

    pub fn into_media(self) -> HistorySyncReadMedia<C> {
        HistorySyncReadMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> HistorySyncReadMediaAdmin<C> {
        HistorySyncReadMediaAdmin::new(self.conn)
    }

    pub fn into_profile(self) -> HistorySyncReadProfile<C> {
        HistorySyncReadProfile::new(self.conn)
    }

    pub fn into_profile_admin(self) -> HistorySyncReadProfileAdmin<C> {
        HistorySyncReadProfileAdmin::new(self.conn)
    }

    pub fn into_chat(self) -> HistorySyncReadChat<C> {
        HistorySyncReadChat::new(self.conn)
    }

    pub fn into_chat_admin(self) -> HistorySyncReadChatAdmin<C> {
        HistorySyncReadChatAdmin::new(self.conn)
    }
}
