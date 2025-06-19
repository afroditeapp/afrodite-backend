use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{NewsId, NewsTranslations};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountNewsAdmin);

impl ReadCommandsAccountNewsAdmin<'_> {
    pub async fn news_translations(&self, id: NewsId) -> Result<NewsTranslations, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds.account_admin().news().news_translations(id)?;
            Ok(value)
        })
        .await
        .into_error()
    }
}
