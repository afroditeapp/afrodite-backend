use account_admin::CurrentWriteAccountAdmin;
use chat::CurrentWriteChatUtils;
use database::DbWriteAccessProvider;

use self::account::CurrentWriteAccount;

pub mod account;
pub mod account_admin;
pub mod chat;

pub trait GetDbWriteCommandsAccount {
    fn account(&mut self) -> CurrentWriteAccount<'_>;
    fn account_admin(&mut self) -> CurrentWriteAccountAdmin<'_>;
    fn account_chat_utils(&mut self) -> CurrentWriteChatUtils<'_>;

}

impl <I: DbWriteAccessProvider> GetDbWriteCommandsAccount for I {
    fn account(&mut self) -> CurrentWriteAccount<'_> {
        CurrentWriteAccount::new(self.handle())
    }

    fn account_admin(&mut self) -> CurrentWriteAccountAdmin<'_> {
        CurrentWriteAccountAdmin::new(self.handle())
    }

    fn account_chat_utils(&mut self) -> CurrentWriteChatUtils<'_> {
        CurrentWriteChatUtils::new(self.handle())
    }
}
