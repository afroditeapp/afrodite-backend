use model::{AccountIdInternal, NewsIteratorSessionIdInternal};
use server_data::{
    cache::CacheError, define_server_data_write_commands, result::Result, write::WriteCommandsProvider, DataError, IntoDataError
};

define_server_data_write_commands!(WriteCommandsAccountNews);
define_db_transaction_command!(WriteCommandsAccountNews);
define_db_read_command_for_write!(WriteCommandsAccountNews);

impl<C: WriteCommandsProvider> WriteCommandsAccountNews<C> {
    pub async fn handle_reset_news_iterator(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<NewsIteratorSessionIdInternal, DataError> {
        let latest_used_id = self.db_read(|mut cmds| cmds.account().news().latest_used_news_id()).await?;
        let session_id = self.cache()
            .write_cache(id.as_id(), |e| {
                if let Some(c) = e.account.as_mut() {
                    Ok(c.news_iterator.reset(latest_used_id))
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
            .await
            .into_data_error(id)?;

        Ok(session_id)
    }

    pub async fn reset_news_count_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().news().reset_news_count_sync_version(id)
        })
    }
}
