use model::{AccountIdInternal, NewsCountResult, NewsId, NewsItemSimple};
use server_data::{
    cache::db_iterator::DbIteratorState, define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError
};

define_server_data_read_commands!(ReadCommandsAccountNews);
define_db_read_command!(ReadCommandsAccountNews);

impl<C: ReadCommandsProvider> ReadCommandsAccountNews<C> {
    pub async fn news_count(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<NewsCountResult, DataError> {
        self
            .db_read(move |mut cmds| {
                let c = cmds.account().news().news_count()?;
                let v = cmds.account().news().news_sync_version(id)?;
                Ok(NewsCountResult {v, c})
            })
            .await
            .into_data_error(id)
    }

    pub async fn news_page(
        &self,
        state: DbIteratorState<NewsId>,
    ) -> Result<Vec<NewsItemSimple>, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds
                .account()
                .news()
                .paged_news(
                    state.id_at_reset(),
                    state.page().try_into().unwrap_or(i64::MAX),
                )?;
            Ok(value)
        })
        .await
        .into_error()
    }
}
