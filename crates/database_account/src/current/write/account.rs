use database::{define_current_write_commands, ConnectionProvider};

mod data;
mod demo;
mod sign_in_with;
mod email;

define_current_write_commands!(CurrentWriteAccount, CurrentSyncWriteAccount);

impl<C: ConnectionProvider> CurrentSyncWriteAccount<C> {
    pub fn data(self) -> data::CurrentSyncWriteAccountData<C> {
        data::CurrentSyncWriteAccountData::new(self.cmds)
    }

    pub fn sign_in_with(self) -> sign_in_with::CurrentSyncWriteAccountSignInWith<C> {
        sign_in_with::CurrentSyncWriteAccountSignInWith::new(self.cmds)
    }

    pub fn demo_mode(self) -> demo::CurrentSyncWriteAccountDemo<C> {
        demo::CurrentSyncWriteAccountDemo::new(self.cmds)
    }

    pub fn email(self) -> email::CurrentSyncWriteAccountEmail<C> {
        email::CurrentSyncWriteAccountEmail::new(self.cmds)
    }
}
