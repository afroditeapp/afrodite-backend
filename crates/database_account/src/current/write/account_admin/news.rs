
use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{delete, insert_into, prelude::*, update, upsert::excluded};
use error_stack::Result;
use model::{
    AccountIdInternal, NewsId, NewsLocale, SyncVersion, UnixTime, UpdateNewsTranslation
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountNewsAdmin, CurrentSyncWriteAccountNewsAdmin);

impl<C: ConnectionProvider> CurrentSyncWriteAccountNewsAdmin<C> {
    pub fn create_new_news_item(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<NewsId, DieselDatabaseError> {
        use model::schema::news::dsl::*;

        let news_id_value: NewsId = insert_into(news)
            .values((
                account_id_creator.eq(id_value.as_db_id()),
            ))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(news_id_value)
    }

    pub fn delete_news_item(
        &mut self,
        id_value: NewsId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::news::dsl::*;

        delete(news)
            .filter(id.eq(id_value))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_news_translation(
        &mut self,
        id_value: AccountIdInternal,
        news_id_value: NewsId,
        locale_value: NewsLocale,
        content: UpdateNewsTranslation,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::news_translations::dsl::*;

        let current_time = UnixTime::current_time();

        insert_into(news_translations)
            .values((
                locale.eq(locale_value.locale),
                news_id.eq(news_id_value),
                title.eq(content.title),
                body.eq(content.body),
                creation_unix_time.eq(current_time),
                account_id_creator.eq(id_value.as_db_id()),
            ))
            .on_conflict((news_id, locale))
            .do_update()
            .set((
                title.eq(excluded(title)),
                body.eq(excluded(body)),
                version_number.eq(version_number + 1),
                edit_unix_time.eq(current_time),
                account_id_editor.eq(id_value.as_db_id()),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn delete_news_translation(
        &mut self,
        id_value: NewsId,
        locale_value: NewsLocale,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::news_translations::dsl::*;

        delete(news_translations)
            .filter(news_id.eq(id_value))
            .filter(locale.eq(locale_value.locale))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn set_news_publicity(
        &mut self,
        id_value: NewsId,
        is_public: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::news::dsl::*;

        let current_value = self.read().account_admin().news().news_translations(id_value)?;
        let current_time = UnixTime::current_time();
        let first_publication = if is_public && current_value.first_publication_time.is_none() {
            Some(current_time)
        } else {
            current_value.first_publication_time
        };
        let latest_publication = if is_public {
            Some(current_time)
        } else {
            current_value.latest_publication_time
        };

        update(news)
            .filter(id.eq(id_value))
            .set((
                public.eq(is_public),
                first_publication_unix_time.eq(first_publication),
                latest_publication_unix_time.eq(latest_publication),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        self.increment_news_count_sync_version_for_every_account()?;

        Ok(())
    }

    fn increment_news_count_sync_version_for_every_account(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(news_count_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(news_count_sync_version.eq(news_count_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
