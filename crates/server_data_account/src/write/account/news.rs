use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model_account::{AccountIdInternal, NewsIteratorState, ResetNewsIteratorResult};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountNews);

impl WriteCommandsAccountNews<'_> {
    pub async fn handle_reset_news_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<ResetNewsIteratorResult, DataError> {
        let (new_id, v, count) = db_transaction!(self, move |mut cmds| {
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
            Ok((new_id, v, c))
        })?;

        Ok(ResetNewsIteratorResult {
            s: NewsIteratorState {
                id_at_reset: new_id,
                page: 0,
            },
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
