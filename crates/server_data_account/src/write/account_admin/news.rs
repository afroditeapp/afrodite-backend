use model::{AccountIdInternal, NewsId};
use server_data::{
    define_server_data_write_commands, result::Result, write::WriteCommandsProvider, DataError,
};

define_server_data_write_commands!(WriteCommandsAccountNewsAdmin);
define_db_transaction_command!(WriteCommandsAccountNewsAdmin);

impl<C: WriteCommandsProvider> WriteCommandsAccountNewsAdmin<C> {
    pub async fn create_news_item(
        &self,
        id: AccountIdInternal,
    ) -> Result<NewsId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().create_new_news_item(id)
        })
    }

    pub async fn delete_news_item(
        &self,
        id: NewsId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().delete_news_item(id)
        })
    }
}
