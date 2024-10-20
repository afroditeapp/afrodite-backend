use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, AccountIdInternal, NewsId, NewsItem, NewsItemInternal, NewsItemSimple, NewsLocale, NewsSyncVersion, NewsTranslationInternal, RequireNewsLocale};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountNews, CurrentSyncReadAccountNews);

impl<C: ConnectionProvider> CurrentSyncReadAccountNews<C> {
    pub fn latest_used_news_id(
        &mut self,
    ) -> Result<Option<NewsId>, DieselDatabaseError> {
        use crate::schema::news::dsl::*;

        news
            .select(id)
            .limit(1)
            .order(id.desc())
            .first(self.conn())
            .optional()
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
        locale_value: NewsLocale,
        include_private_news: bool,
    ) -> Result<Vec<NewsItemSimple>, DieselDatabaseError> {
        use crate::schema::{news, news_translations};

        const PAGE_SIZE: i64 = 25;

        let include_private_news = diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(include_private_news);

        let rows: Vec<(NewsItemInternal, Option<String>)> = news::table
            .left_outer_join(
                news_translations::table.on(
                    news::id.eq(news_translations::news_id).and(
                        news_translations::locale.eq(locale_value.locale).or(news_translations::locale.eq(NewsLocale::ENGLISH))
                    )
                ),
            )
            .filter(news::public.eq(true).or(include_private_news))
            .filter(news::id.le(news_id_value))
            .select((
                NewsItemInternal::as_select(),
                news_translations::title.nullable(),
            ))
            .order((
                news::id.desc(),
            ))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        let items = rows
            .into_iter()
            .map(|r| {
                NewsItemSimple {
                    id: r.0.id,
                    title: r.1,
                    time: r.0.first_publication_unix_time,
                    private: !r.0.public
                }
            })
            .collect();

        Ok(items)
    }

    pub fn news_item(
        &mut self,
        news_id_value: NewsId,
        locale_value: NewsLocale,
        require_locale: RequireNewsLocale,
    ) -> Result<Option<NewsItem>, DieselDatabaseError> {
        use crate::schema::{account_id, news, news_translations};

        let (creator_aid, editor_aid) = alias!(account_id as creator_aid, account_id as editor_aid);

        let value: Option<(NewsTranslationInternal, Option<AccountId>, Option<AccountId>)> = news::table
            .inner_join(
                news_translations::table.on(
                    news::id.eq(news_translations::news_id).and(
                        news_translations::locale.eq(locale_value.locale.clone()).or(news_translations::locale.eq(NewsLocale::ENGLISH))
                    )
                ),
            )
            .left_outer_join(
                creator_aid.on(
                    news_translations::account_id_creator.assume_not_null().eq(creator_aid.field(account_id::id))
                ),
            )
            .left_outer_join(
                editor_aid.on(
                    news_translations::account_id_editor.assume_not_null().eq(editor_aid.field(account_id::id))
                ),
            )
            .filter(news::id.eq(news_id_value))
            .select((
                NewsTranslationInternal::as_select(),
                creator_aid.field(account_id::uuid).nullable(),
                editor_aid.field(account_id::uuid).nullable()
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        let (internal, creator, editor) = if let Some(value) = value {
            value
        } else {
            return Ok(None);
        };

        if require_locale.require_locale && internal.locale != locale_value.locale {
            return Ok(None);
        }

        let news_item = NewsItem {
            title: internal.title,
            body: internal.body,
            locale: internal.locale,
            creation_time: internal.creation_unix_time,
            version: Some(internal.version_number),
            aid_creator: creator,
            aid_editor: editor,
            edit_unix_time: internal.edit_unix_time.map(|x| x.ut),
        };

        Ok(Some(news_item))
    }
}
