use database::define_current_read_commands;

define_current_read_commands!(CurrentReadAccount);

mod ban;
mod client_features;
mod data;
mod delete;
mod demo;
mod email;
mod news;
mod notification;
mod report;
mod sign_in_with;

impl<'a> CurrentReadAccount<'a> {
    pub fn ban(self) -> ban::CurrentReadAccountBan<'a> {
        ban::CurrentReadAccountBan::new(self.cmds)
    }

    pub fn data(self) -> data::CurrentReadAccountData<'a> {
        data::CurrentReadAccountData::new(self.cmds)
    }

    pub fn delete(self) -> delete::CurrentReadAccountDelete<'a> {
        delete::CurrentReadAccountDelete::new(self.cmds)
    }

    pub fn sign_in_with(self) -> sign_in_with::CurrentReadAccountSignInWith<'a> {
        sign_in_with::CurrentReadAccountSignInWith::new(self.cmds)
    }

    pub fn demo(self) -> demo::CurrentReadAccountDemo<'a> {
        demo::CurrentReadAccountDemo::new(self.cmds)
    }

    pub fn email(self) -> email::CurrentReadAccountEmail<'a> {
        email::CurrentReadAccountEmail::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentReadAccountNews<'a> {
        news::CurrentReadAccountNews::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentReadAccountReport<'a> {
        report::CurrentReadAccountReport::new(self.cmds)
    }

    pub fn client_features(self) -> client_features::CurrentReadAccountClientFeatures<'a> {
        client_features::CurrentReadAccountClientFeatures::new(self.cmds)
    }

    pub fn notification(self) -> notification::CurrentReadAccountNotification<'a> {
        notification::CurrentReadAccountNotification::new(self.cmds)
    }
}
