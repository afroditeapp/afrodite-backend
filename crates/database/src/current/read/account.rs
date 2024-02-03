



use simple_backend_database::{
    diesel_db::{ConnectionProvider},
};




define_read_commands!(CurrentReadAccount, CurrentSyncReadAccount);

mod data;
mod sign_in_with;
mod token;

impl CurrentReadAccount<'_> {
    pub fn data(&self) -> data::CurrentReadAccountData {
        data::CurrentReadAccountData::new(self.cmds)
    }
}

impl<C: ConnectionProvider> CurrentSyncReadAccount<C> {
    pub fn data(self) -> data::CurrentSyncReadAccountData<C> {
        data::CurrentSyncReadAccountData::new(self.cmds)
    }

    pub fn sign_in_with(self) -> sign_in_with::CurrentSyncReadAccountSignInWith<C> {
        sign_in_with::CurrentSyncReadAccountSignInWith::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentSyncReadAccountToken<C> {
        token::CurrentSyncReadAccountToken::new(self.cmds)
    }
}
