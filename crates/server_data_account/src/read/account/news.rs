use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{
    AccountIdInternal, NewsId, NewsItem, NewsItemSimple, NewsLocale, PageItemCountForNewPublicNews,
    PublicationId, RequireNewsLocale, UnreadNewsCountResult,
};
use server_data::{
    cache::db_iterator::new_count::DbIteratorStateNewCount, define_cmd_wrapper_read, read::DbRead,
    result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsAccountNews);

impl ReadCommandsAccountNews<'_> {
    pub async fn unread_news_count(
        &self,
        id: AccountIdInternal,
    ) -> Result<UnreadNewsCountResult, DataError> {
        self.db_read(move |mut cmds| {
            let c = cmds.account().news().unread_news_count(id)?;
            let v = cmds.account().news().news_sync_version(id)?;
            Ok(UnreadNewsCountResult { v, c })
        })
        .await
        .into_data_error(id)
    }

    pub async fn news_page(
        &self,
        state: DbIteratorStateNewCount<PublicationId>,
        locale: NewsLocale,
        include_private_news: bool,
    ) -> Result<(Vec<NewsItemSimple>, PageItemCountForNewPublicNews), DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds.account().news().paged_news(
                state.id_at_reset(),
                state.previous_id_at_reset(),
                state.page().try_into().unwrap_or(i64::MAX),
                locale,
                include_private_news,
            )?;
            Ok(value)
        })
        .await
        .into_error()
    }

    pub async fn news_item(
        &self,
        id: NewsId,
        locale: NewsLocale,
        require_locale: RequireNewsLocale,
    ) -> Result<Option<NewsItem>, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds
                .account()
                .news()
                .news_item(id, locale, require_locale)?;
            Ok(value)
        })
        .await
        .into_error()
    }

    pub async fn is_public(&self, id: NewsId) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds.account().news().is_public(id)?;
            Ok(value)
        })
        .await
        .into_error()
    }
}
