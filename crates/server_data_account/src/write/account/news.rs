use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model_account::{AccountIdInternal, ResetNewsIteratorResult};
use server_data::{
    define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError, IntoDataError,
};

use crate::cache::CacheWriteAccount;

define_cmd_wrapper_write!(WriteCommandsAccountNews);

impl WriteCommandsAccountNews<'_> {
    pub async fn handle_reset_news_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<ResetNewsIteratorResult, DataError> {
        let (previous_id, new_id, v, count) = db_transaction!(self, move |mut cmds| {
            let previous_id = cmds
                .read()
                .account()
                .news()
                .publication_id_at_news_iterator_reset(id)?;
            let new_id = cmds
                .read()
                .account()
                .data()
                .global_state()?
                .next_news_publication_id
                .to_latest_used_id();
            let v = cmds
                .account()
                .news()
                .increment_news_sync_version_for_specific_account(id)?;
            let c = cmds.account().news().reset_news_unread_count(id)?;
            Ok((previous_id, new_id, v, c))
        })?;

        let session_id = self
            .write_cache_account(id.as_id(), |e| {
                Ok(e.news_iterator.reset(new_id, previous_id))
            })
            .await
            .into_data_error(id)?;

        Ok(ResetNewsIteratorResult {
            s: session_id.into(),
            v,
            c: count,
        })
    }

    pub async fn reset_news_count_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().news().reset_news_sync_version(id)
        })
    }
}
