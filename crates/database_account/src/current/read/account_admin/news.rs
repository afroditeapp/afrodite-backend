use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, NewsId, NewsItem, NewsTranslationInternal, NewsTranslations};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountNewsAdmin, CurrentSyncReadAccountNewsAdmin);

impl<C: ConnectionProvider> CurrentSyncReadAccountNewsAdmin<C> {
    pub fn news_translations(
        &mut self,
        news_id_value: NewsId,
    ) -> Result<NewsTranslations, DieselDatabaseError> {
        use crate::schema::{account_id, news, news_translations};

        let (is_news_public, news_creator) = {
            let creator_aid = alias!(account_id as creator_aid);
            news::table
                .left_outer_join(
                    creator_aid.on(
                        news::account_id_creator.assume_not_null().eq(creator_aid.field(account_id::id))
                    ),
                )
                .filter(news::id.eq(news_id_value))
                .select((
                    news::public,
                    creator_aid.field(account_id::uuid).nullable(),
                ))
                .first(self.conn())
                .into_db_error(())?
        };

        let translations: Vec<(NewsTranslationInternal, Option<AccountId>, Option<AccountId>)> = {
            let (creator_aid, editor_aid) = alias!(account_id as creator_aid, account_id as editor_aid);
            news_translations::table
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
                .filter(news_translations::news_id.eq(news_id_value))
                .select((
                    NewsTranslationInternal::as_select(),
                    creator_aid.field(account_id::uuid).nullable(),
                    editor_aid.field(account_id::uuid).nullable()
                ))
                .load(self.conn())
                .into_db_error(())?
        };

        let translations: Vec<NewsItem> = translations
            .into_iter()
            .map(|(internal, creator, editor)| {
                NewsItem {
                    title: internal.title,
                    body: internal.body,
                    locale: internal.locale,
                    creation_time: internal.creation_unix_time,
                    aid_creator: creator,
                    aid_editor: editor,
                    edit_time: internal.edit_unix_time,
                }
            })
            .collect();

        Ok(NewsTranslations {
            id: news_id_value,
            public: is_news_public,
            aid_creator: news_creator,
            translations,
        })
    }
}
