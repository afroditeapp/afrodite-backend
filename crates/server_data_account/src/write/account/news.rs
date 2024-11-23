use model_account::{AccountIdInternal, ResetNewsIteratorResult};
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
    ) -> Result<ResetNewsIteratorResult, DataError> {
        let (previous_id, new_id, v, c) = db_transaction!(self, move |mut cmds| {
            let previous_id = cmds.read().account().news().publication_id_at_news_iterator_reset(id)?;
            let new_id = cmds.read().account().data().global_state()?
                .next_news_publication_id
                .to_latest_used_id();
            let v = cmds.account().news().increment_news_sync_version_for_specific_account(id)?;
            let c = cmds.account().news().reset_news_unread_count(id)?;
            Ok((previous_id, new_id, v, c))
        })?;

        let session_id = self.cache()
            .write_cache(id.as_id(), |e| {
                if let Some(c) = e.account.as_mut() {
                    Ok(c.news_iterator.reset(new_id, previous_id))
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
            .await
            .into_data_error(id)?;

        Ok(ResetNewsIteratorResult {
            s: session_id.into(),
            v,
            c
        })
    }

    pub async fn reset_news_count_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().news().reset_news_sync_version(id)
        })
    }
}
