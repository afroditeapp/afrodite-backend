use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update, upsert::excluded};
use error_stack::Result;
use model::{AccountIdInternal, SyncVersion, UnixTime};
use model_account::{AccountGlobalState, NewsId, NewsLocale, PublicationId, UpdateNewsTranslation};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsAccount};

define_current_write_commands!(CurrentWriteAccountNewsAdmin);

impl CurrentWriteAccountNewsAdmin<'_> {
    pub fn create_new_news_item(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<NewsId, DieselDatabaseError> {
        use model::schema::news::dsl::*;

        let news_id_value: NewsId = insert_into(news)
            .values((account_id_creator.eq(id_value.as_db_id()),))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(news_id_value)
    }

    pub fn delete_news_item(&mut self, id_value: NewsId) -> Result<(), DieselDatabaseError> {
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
            .execute_my_conn(self.conn())
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

    /// Returns Ok(true) if news notification should be sent to clients.
    pub fn set_news_publicity(
        &mut self,
        id_value: NewsId,
        is_public: bool,
    ) -> Result<bool, DieselDatabaseError> {
        use model::schema::news::dsl::*;

        let current_value = self
            .read()
            .account_admin()
            .news()
            .news_translations(id_value)?;
        let current_time = UnixTime::current_time();
        let first_publication = if is_public && current_value.first_publication_time.is_none() {
            Some(current_time)
        } else {
            current_value.first_publication_time
        };
        let (publication_id_value, latest_publication, send_notification) = if is_public {
            let new_publication_id = self.get_next_news_publication_id_and_increment_it()?;
            self.increment_news_unread_count_for_every_account()?;
            self.increment_news_sync_version_for_every_account()?;
            (Some(new_publication_id), Some(current_time), true)
        } else {
            let publication_id_value: PublicationId = news
                .filter(id.eq(id_value))
                .filter(publication_id.is_not_null())
                .select(publication_id.assume_not_null())
                .first(self.conn())
                .into_db_error(())?;
            let latest_used_publication_id = self
                .read()
                .account_admin()
                .news()
                .get_next_news_publication_id()?
                .to_latest_used_id();
            // Limit decrementing to only latest publication ID to
            // prevent the following case:
            // - Publish news A   - unread news 1
            // - View news        - unread news 0
            // - Publish news B   - unread news 1
            // - Unpublish news A - unread news 0
            let send_notification = if publication_id_value == latest_used_publication_id {
                self.decrement_news_unread_count_for_every_account()?;
                self.increment_news_sync_version_for_every_account()?;
                true
            } else {
                false
            };
            (
                None,
                current_value.latest_publication_time,
                send_notification,
            )
        };

        update(news)
            .filter(id.eq(id_value))
            .set((
                publication_id.eq(publication_id_value),
                first_publication_unix_time.eq(first_publication),
                latest_publication_unix_time.eq(latest_publication),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(send_notification)
    }

    fn increment_news_sync_version_for_every_account(&mut self) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(news_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(news_sync_version.eq(news_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    fn increment_news_unread_count_for_every_account(&mut self) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .set(unread_news_count.eq(unread_news_count + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    fn decrement_news_unread_count_for_every_account(&mut self) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(unread_news_count.gt(0))
            .set(unread_news_count.eq(unread_news_count - 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn get_next_news_publication_id_and_increment_it(
        &mut self,
    ) -> Result<PublicationId, DieselDatabaseError> {
        use model::schema::account_global_state::dsl::*;

        let id = self
            .read()
            .account()
            .data()
            .global_state()?
            .next_news_publication_id;

        insert_into(account_global_state)
            .values((
                row_type.eq(AccountGlobalState::ACCOUNT_GLOBAL_STATE_ROW_TYPE),
                next_news_publication_id.eq(1),
            ))
            .on_conflict(row_type)
            .do_update()
            .set(next_news_publication_id.eq(next_news_publication_id + 1))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(id)
    }
}
