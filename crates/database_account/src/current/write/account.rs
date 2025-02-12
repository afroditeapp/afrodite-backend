use database::define_current_write_commands;

mod data;
mod delete;
mod demo;
mod email;
mod news;
mod sign_in_with;
mod report;

define_current_write_commands!(CurrentWriteAccount);

impl<'a> CurrentWriteAccount<'a> {
    pub fn data(self) -> data::CurrentWriteAccountData<'a> {
        data::CurrentWriteAccountData::new(self.cmds)
    }

    pub fn delete(self) -> delete::CurrentWriteAccountDelete<'a> {
        delete::CurrentWriteAccountDelete::new(self.cmds)
    }

    pub fn sign_in_with(self) -> sign_in_with::CurrentWriteAccountSignInWith<'a> {
        sign_in_with::CurrentWriteAccountSignInWith::new(self.cmds)
    }

    pub fn demo_mode(self) -> demo::CurrentWriteAccountDemo<'a> {
        demo::CurrentWriteAccountDemo::new(self.cmds)
    }

    pub fn email(self) -> email::CurrentWriteAccountEmail<'a> {
        email::CurrentWriteAccountEmail::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentWriteAccountNews<'a> {
        news::CurrentWriteAccountNews::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentWriteAccountReport<'a> {
        report::CurrentWriteAccountReport::new(self.cmds)
    }
}
