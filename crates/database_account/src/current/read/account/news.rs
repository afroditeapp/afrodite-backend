use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, AccountIdInternal, NewsCount, NewsId, NewsItem, NewsItemInternal, NewsItemSimple, NewsSyncVersion};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountNews, CurrentSyncReadAccountNews);

impl<C: ConnectionProvider> CurrentSyncReadAccountNews<C> {
    pub fn news_count(
        &mut self,
    ) -> Result<NewsCount, DieselDatabaseError> {
        use crate::schema::news::dsl::*;

        news
            .count()
            .get_result(self.conn())
            .into_db_error(())
    }

    pub fn latest_used_news_id(
        &mut self,
    ) -> Result<NewsId, DieselDatabaseError> {
        use crate::schema::news::dsl::*;

        news
            .select(id)
            .limit(1)
            .order(id.desc())
            .first(self.conn())
            .into_db_error(())
    }

    pub fn news_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<NewsSyncVersion, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(news_count_sync_version)
            .first(self.conn())
            .into_db_error(id)
    }

    /// News ordering goes from recent to older starting
    /// from `news_id_value`.
    pub fn paged_news(
        &mut self,
        news_id_value: NewsId,
        page: i64,
    ) -> Result<Vec<NewsItemSimple>, DieselDatabaseError> {
        use crate::schema::news::dsl::*;

        const PAGE_SIZE: i64 = 25;

        let account_ids: Vec<NewsItemSimple> = news
            .filter(id.is_not_null())
            .filter(id.le(news_id_value))
            .select(NewsItemSimple::as_select())
            .order((
                id.desc(),
            ))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        Ok(account_ids)
    }

    pub fn news_item(
        &mut self,
        news_id_value: NewsId,
    ) -> Result<NewsItem, DieselDatabaseError> {
        use crate::schema::{account_id, news::dsl::*};

        let (creator_aid, editor_aid) = alias!(account_id as creator_aid, account_id as editor_aid);

        let (internal, creator, editor): (NewsItemInternal, Option<AccountId>, Option<AccountId>) = news
            .left_outer_join(
                creator_aid.on(
                    account_id_creator.assume_not_null().eq(creator_aid.field(account_id::id))
                ),
            )
            .left_outer_join(
                editor_aid.on(
                    account_id_editor.assume_not_null().eq(editor_aid.field(account_id::id))
                ),
            )
            .filter(id.eq(news_id_value))
            .select((
                NewsItemInternal::as_select(),
                creator_aid.field(account_id::uuid).nullable(),
                editor_aid.field(account_id::uuid).nullable()
            ))
            .first(self.conn())
            .into_db_error(())?;

        let news_item = NewsItem {
            title: internal.title,
            body: internal.body,
            creation_time: internal.creation_unix_time,
            aid_creator: creator,
            aid_editor: editor,
            edit_time: internal.edit_unix_time,
        };

        Ok(news_item)
    }
}
