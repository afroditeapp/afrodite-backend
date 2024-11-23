use model_account::{NewsId, NewsTranslations};
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError
};

define_server_data_read_commands!(ReadCommandsAccountNews);
define_db_read_command!(ReadCommandsAccountNews);

impl<C: ReadCommandsProvider> ReadCommandsAccountNews<C> {
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
