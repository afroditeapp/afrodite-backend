use sqlx::{SqlitePool};

use self::{
    account::{CurrentSyncWriteAccount, CurrentWriteAccount},
    chat::{CurrentSyncWriteChat, CurrentWriteChat},
    media::{CurrentSyncWriteMedia, CurrentWriteMedia},
    media_admin::{CurrentWriteMediaAdmin, CurrentSyncWriteMediaAdmin},
    profile::{CurrentSyncWriteProfile, CurrentWriteProfile},
};
use crate::{diesel::{DieselConnection, DieselDatabaseError}, sqlite::CurrentDataWriteHandle, TransactionError};

macro_rules! define_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::current::write::CurrentWriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::current::write::CurrentWriteCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn read(&self) -> crate::current::read::SqliteReadCommands<'a> {
                self.cmds.handle.read()
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<'a> {
            cmds: crate::current::write::CurrentSyncWriteCommands<'a>,
        }

        impl<'a> $sync_name<'a> {
            pub fn new(cmds: crate::current::write::CurrentSyncWriteCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn conn(&'a mut self) -> &'a mut crate::diesel::DieselConnection {
                &mut self.cmds.conn
            }

            pub fn into_conn(self) -> &'a mut crate::diesel::DieselConnection {
                self.cmds.conn
            }

            pub fn read(conn: &mut crate::diesel::DieselConnection) -> crate::current::read::CurrentSyncReadCommands<'_> {
                crate::current::read::CurrentSyncReadCommands::new(conn)
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

#[derive(Clone, Debug)]
pub struct CurrentWriteCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub fn account(&'a self) -> CurrentWriteAccount<'a> {
        CurrentWriteAccount::new(self)
    }

    pub fn media(&'a self) -> CurrentWriteMedia<'a> {
        CurrentWriteMedia::new(self)
    }

    pub fn media_admin(&'a self) -> CurrentWriteMediaAdmin<'a> {
        CurrentWriteMediaAdmin::new(self)
    }

    pub fn profile(&'a self) -> CurrentWriteProfile<'a> {
        CurrentWriteProfile::new(self)
    }

    pub fn chat(&'a self) -> CurrentWriteChat<'a> {
        CurrentWriteChat::new(self)
    }

    pub fn pool(&'a self) -> &SqlitePool {
        self.handle.pool()
    }
}

pub struct CurrentSyncWriteCommands<'a> {
    conn: &'a mut DieselConnection,
}

impl<'a> CurrentSyncWriteCommands<'a> {
    pub fn new(conn: &'a mut DieselConnection) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> CurrentSyncWriteAccount<'a> {
        CurrentSyncWriteAccount::new(self)
    }

    pub fn into_media(self) -> CurrentSyncWriteMedia<'a> {
        CurrentSyncWriteMedia::new(self)
    }

    pub fn into_media_admin(self) -> CurrentSyncWriteMediaAdmin<'a> {
        CurrentSyncWriteMediaAdmin::new(self)
    }

    pub fn into_profile(self) -> CurrentSyncWriteProfile<'a> {
        CurrentSyncWriteProfile::new(self)
    }

    pub fn account(&'a mut self) -> CurrentSyncWriteAccount<'a> {
        CurrentSyncWriteAccount::new(self.write())
    }

    pub fn media(&'a mut self) -> CurrentSyncWriteMedia<'a> {
        CurrentSyncWriteMedia::new(self.write())
    }

    pub fn media_admin(&'a mut self) -> CurrentSyncWriteMediaAdmin<'a> {
        CurrentSyncWriteMediaAdmin::new(self.write())
    }

    pub fn profile(&'a mut self) -> CurrentSyncWriteProfile<'a> {
        CurrentSyncWriteProfile::new(self.write())
    }


    pub fn chat(self) -> CurrentSyncWriteChat<'a> {
        CurrentSyncWriteChat::new(self)
    }

    pub fn read(&mut self) -> crate::current::read::CurrentSyncReadCommands<'_> {
        crate::current::read::CurrentSyncReadCommands::new(self.conn)
    }

    pub fn write(&'a mut self) -> crate::current::write::CurrentSyncWriteCommands<'a> {
        Self::new(self.conn)
    }

    pub fn conn(&'a mut self) -> &'a mut DieselConnection {
        self.conn
    }

    pub fn transaction<
        F: FnOnce(&mut DieselConnection) -> std::result::Result<T, TransactionError<DieselDatabaseError>> + 'static,
        T,
    >(self, transaction_actions: F) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        Ok(self.conn.transaction(transaction_actions)?)
    }
}

pub struct TransactionConnection<'a> {
    pub conn: &'a mut DieselConnection,
}

// pub trait WriteCmdsMethods<'a: 'b, 'b>: Sized {
//     fn conn(self) -> &'b mut DieselConnection;
//     fn conn_ref_mut(&'a mut self) -> &'b mut DieselConnection;

//     // fn write_ref(&'a self) -> crate::current::write::CurrentSyncWriteCommands<'a>;

//     fn write(self) -> crate::current::write::CurrentSyncWriteCommands<'b> {
//         CurrentSyncWriteCommands::new(self.conn())
//     }

//     fn write_ref_mut(&'a mut self) -> crate::current::write::CurrentSyncWriteCommands<'b> {
//         CurrentSyncWriteCommands::new(self.conn_ref_mut())
//     }

//     fn into_account(self) -> CurrentSyncWriteAccount<'b> {
//         CurrentSyncWriteAccount::new(self.write())
//     }

//     fn into_media(self) -> CurrentSyncWriteMedia<'b> {
//         CurrentSyncWriteMedia::new(self.write())
//     }

//     fn into_media_admin(self) -> CurrentSyncWriteMediaAdmin<'b> {
//         CurrentSyncWriteMediaAdmin::new(self.write())
//     }

//     fn into_profile(self) -> CurrentSyncWriteProfile<'b> {
//         CurrentSyncWriteProfile::new(self.write())
//     }

//     fn account(&'a mut self) -> CurrentSyncWriteAccount<'b> {
//         CurrentSyncWriteAccount::new(self.write_ref_mut())
//     }

//     fn media(&'a mut self) -> CurrentSyncWriteMedia<'b> {
//         CurrentSyncWriteMedia::new(self.write_ref_mut())
//     }

//     fn media_admin(&'a mut self) -> CurrentSyncWriteMediaAdmin<'b> {
//         CurrentSyncWriteMediaAdmin::new(self.write_ref_mut())
//     }

//     fn profile(&'a mut self) -> CurrentSyncWriteProfile<'b> {
//         CurrentSyncWriteProfile::new(self.write_ref_mut())
//     }
// }

pub trait WriteCmdsMethods<'a, 'b: 'a>: Sized {
    // fn conn(self) -> &'b mut DieselConnection;

    // type R: WriteCmdsMethods<'a, 'b>;

    // fn r(self) -> Self::R;
    // fn r1(conn: &'b mut DieselConnection) -> Self::R;
    fn write(self) -> crate::current::write::CurrentSyncWriteCommands<'b>;

    fn into_account(self) -> CurrentSyncWriteAccount<'b> {
        CurrentSyncWriteAccount::new(self.write())
    }

    fn into_media(self) -> CurrentSyncWriteMedia<'b> {
        CurrentSyncWriteMedia::new(self.write())
    }

    fn into_media_admin(self) -> CurrentSyncWriteMediaAdmin<'b> {
        CurrentSyncWriteMediaAdmin::new(self.write())
    }

    fn into_profile(self) -> CurrentSyncWriteProfile<'b> {
        CurrentSyncWriteProfile::new(self.write())
    }
}


// impl <'a> WriteCmdsMethods<'a> for TransactionConnection<'a> {
//     fn conn(self) -> &'a mut DieselConnection {
//         self.conn
//     }
//     fn conn_ref_mut(&'a mut self) -> &'a mut DieselConnection {
//         &mut self.conn
//     }

//     // fn write_ref(&'a self) -> crate::current::write::CurrentSyncWriteCommands<'a> {
//     //     CurrentSyncWriteCommands { conn: self.conn }
//     // }
// }

impl <'a, 'b: 'a> WriteCmdsMethods<'a, 'b> for TransactionConnection<'b> {
    //type R = TransactionConnection<'b>;
    // fn r(self) -> Self::R {
    //     self
    // }
    // fn r1(conn: &'b mut DieselConnection) -> Self::R {
    //     TransactionConnection { conn }
    // }
    fn write(self) -> crate::current::write::CurrentSyncWriteCommands<'b> {
        CurrentSyncWriteCommands::new(self.conn)
    }

    // fn write_ref(&'a self) -> crate::current::write::CurrentSyncWriteCommands<'a> {
    //     CurrentSyncWriteCommands { conn: self.conn }
    // }
}

// impl <'a, 'b: 'a> WriteCmdsMethods<'a, 'b> for &'a mut CurrentSyncWriteCommands<'b> {
//     fn conn(self) -> &'b mut DieselConnection {
//         self.conn
//     }
//     // fn write_ref(&'a self) -> crate::current::write::CurrentSyncWriteCommands<'a> {
//     //     CurrentSyncWriteCommands { conn: self.conn }
//     // }
// }

// impl <'a> WriteCmdsMethods<'a> for &'a mut DieselConnection {
//     fn conn(self) -> &'a mut DieselConnection {
//         self
//     }
//     fn conn_ref_mut(&'a mut self) -> &'a mut DieselConnection {
//         self
//     }

//     // fn write_ref(&'a self) -> crate::current::write::CurrentSyncWriteCommands<'a> {
//     //     CurrentSyncWriteCommands { conn: self }
//     // }
// }

impl <'a, 'b: 'a> WriteCmdsMethods<'a, 'b> for &'b mut DieselConnection {
    // type R = &'b mut DieselConnection;
    // fn r(self) -> Self::R {
    //     self
    // }
    fn write(self) -> crate::current::write::CurrentSyncWriteCommands<'b> {
        CurrentSyncWriteCommands::new(self)
    }
}


// impl TransactionConnection<'_> {
//     pub fn new(conn: &mut DieselConnection) -> Self {
//         Self { conn }
//     }

//     pub fn conn(&mut self) -> &mut DieselConnection {
//         self.conn
//     }

//     pub fn account(&mut self) -> CurrentSyncWriteAccount<'a> {
//         CurrentSyncWriteAccount::new(self)
//     }

//     pub fn media(self) -> CurrentSyncWriteMedia<'a> {
//         CurrentSyncWriteMedia::new(self)
//     }

//     pub fn media_admin(self) -> CurrentSyncWriteMediaAdmin<'a> {
//         CurrentSyncWriteMediaAdmin::new(self)
//     }

//     pub fn profile(self) -> CurrentSyncWriteProfile<'a> {
//         CurrentSyncWriteProfile::new(self)
//     }

//     pub fn chat(self) -> CurrentSyncWriteChat<'a> {
//         CurrentSyncWriteChat::new(self)
//     }
// }
