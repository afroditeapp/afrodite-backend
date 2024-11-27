use model_account::{NewsId, NewsTranslations};
use server_data::{
    define_cmd_wrapper, result::Result, DataError, IntoDataError
};

use crate::read::DbReadAccount;

define_cmd_wrapper!(ReadCommandsAccountNewsAdmin);

impl<C: DbReadAccount> ReadCommandsAccountNewsAdmin<C> {

    pub async fn news_translations(
        &self,
        id: NewsId,
    ) -> Result<NewsTranslations, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds
                .account_admin()
                .news()
                .news_translations(
                    id,
                )?;
            Ok(value)
        })
        .await
        .into_error()
    }
}
