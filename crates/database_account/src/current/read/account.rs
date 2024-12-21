use database::define_current_read_commands;

define_current_read_commands!(CurrentReadAccount);

mod data;
mod delete;
mod demo;
mod email;
mod news;
mod sign_in_with;

impl<'a> CurrentReadAccount<'a> {
    pub fn data(self) -> data::CurrentReadAccountData<'a> {
        data::CurrentReadAccountData::new(self.cmds)
    }

    pub fn delete(self) -> delete::CurrentReadAccountDelete<'a> {
        delete::CurrentReadAccountDelete::new(self.cmds)
    }

    pub fn sign_in_with(self) -> sign_in_with::CurrentReadAccountSignInWith<'a> {
        sign_in_with::CurrentReadAccountSignInWith::new(self.cmds)
    }

    pub fn demo_mode(self) -> demo::CurrentReadAccountDemo<'a> {
        demo::CurrentReadAccountDemo::new(self.cmds)
    }

    pub fn email(self) -> email::CurrentReadAccountEmail<'a> {
        email::CurrentReadAccountEmail::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentReadAccountNews<'a> {
        news::CurrentReadAccountNews::new(self.cmds)
    }
}
