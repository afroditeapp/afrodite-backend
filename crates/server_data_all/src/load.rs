use database::{
    CurrentReadHandle, DbReaderRaw, DieselDatabaseError, current::read::GetDbReadCommandsCommon,
};
use database_account::current::read::GetDbReadCommandsAccount;
use database_chat::current::read::GetDbReadCommandsChat;
use database_media::current::read::GetDbReadCommandsMedia;
use database_profile::current::read::GetDbReadCommandsProfile;
use error_stack::{Result, ResultExt};
use model::AccountIdInternal;
use server_common::data::WithInfo;
pub use server_common::data::cache::CacheError;
use server_data::{
    cache::{
        CacheEntry, DatabaseCache, account::CacheAccount, chat::CacheChat, common::CacheCommon,
        media::CacheMedia, profile::CacheProfile,
    },
    index::{LocationIndexIteratorHandle, LocationIndexManager, LocationIndexWriteHandle},
};
use tracing::info;

pub struct DbDataToCacheLoader;

impl DbDataToCacheLoader {
    pub async fn load_to_cache(
        cache: &DatabaseCache,
        current_db: &CurrentReadHandle,
        location_index: &LocationIndexManager,
    ) -> Result<(), CacheError> {
        // Load data from database to memory.
        info!("Starting to load data from database to memory");

        let db = DbReaderAll::new(DbReaderRaw::new(current_db));
        let accounts = db
            .db_read(move |mut cmd| cmd.common().account_ids_internal())
            .await
            .change_context(CacheError::Init)?;

        for id in accounts {
            Self::load_account_from_db(
                cache,
                id,
                current_db,
                LocationIndexIteratorHandle::new(location_index),
                LocationIndexWriteHandle::new(location_index),
            )
            .await
            .change_context(CacheError::Init)?;
        }

        info!("Loading to memory complete");
        Ok(())
    }

    pub async fn load_account_from_db(
        cache: &DatabaseCache,
        account_id: AccountIdInternal,
        current_db: &CurrentReadHandle,
        index_iterator: LocationIndexIteratorHandle<'_>,
        index_writer: LocationIndexWriteHandle<'_>,
    ) -> Result<(), CacheError> {
        let db = DbReaderAll::new(DbReaderRaw::new(current_db));
        let login_session = db
            .db_read(move |mut cmds| cmds.common().token().login_session(account_id))
            .await?;

        // Common

        let mut cache_common = CacheCommon::default();
        let permissions = db
            .db_read(move |mut cmds| cmds.common().state().account_permissions(account_id))
            .await?;
        cache_common.permissions = permissions;
        let state = db
            .db_read(move |mut cmds| {
                cmds.common()
                    .state()
                    .account_state_related_shared_state(account_id)
            })
            .await?;
        cache_common.account_state_related_shared_state = state;
        let other_state = db
            .db_read(move |mut cmds| cmds.common().state().other_shared_state(account_id))
            .await?;
        cache_common.other_shared_state = other_state;

        let push_notification_state = db
            .db_read(move |mut cmds| {
                cmds.common()
                    .push_notification()
                    .push_notification_db_state(account_id)
            })
            .await?;
        cache_common.pending_push_notification_flags = push_notification_state.pending_flags.into();
        cache_common.sent_push_notification_flags = push_notification_state.sent_flags.into();

        // App notification settings
        {
            let account = db
                .db_read(move |mut cmds| {
                    cmds.account()
                        .notification()
                        .app_notification_settings(account_id)
                })
                .await?;
            cache_common.app_notification_settings.account = account;
            let profile = db
                .db_read(move |mut cmds| {
                    cmds.profile()
                        .notification()
                        .app_notification_settings(account_id)
                })
                .await?;
            cache_common.app_notification_settings.profile = profile;
            let media = db
                .db_read(move |mut cmds| {
                    cmds.media()
                        .notification()
                        .app_notification_settings(account_id)
                })
                .await?;
            cache_common.app_notification_settings.media = media;
            let chat = db
                .db_read(move |mut cmds| {
                    cmds.chat()
                        .notification()
                        .app_notification_settings(account_id)
                })
                .await?;
            cache_common.app_notification_settings.chat = chat;
        }

        // Account

        let cache_account = CacheAccount::default();

        // Media

        // Media must be before profile because ProfileLink can
        // can contain ProfileContentVersion.
        let media_content = db
            .db_read(move |mut cmds| {
                cmds.media()
                    .media_content()
                    .current_account_media_raw(account_id)
            })
            .await?;
        let media_state = db
            .db_read(move |mut cmds| cmds.media().get_media_state(account_id))
            .await?;
        let cache_media = CacheMedia::new(
            account_id.uuid,
            media_content.profile_content_version_uuid,
            media_state.profile_content_edited_unix_time,
        );

        // Profile

        let profile = db
            .db_read(move |mut cmds| cmds.profile().data().profile_internal(account_id))
            .await?;
        let state = db
            .db_read(move |mut cmds| cmds.profile().data().profile_state(account_id))
            .await?;
        let profile_location = db
            .db_read(move |mut cmds| cmds.profile().data().profile_location(account_id))
            .await?;
        let attributes = db
            .db_read(move |mut cmds| cmds.profile().data().profile_attribute_values(account_id))
            .await?;
        let filters = db
            .db_read(move |mut cmds| cmds.profile().data().profile_attribute_filters(account_id))
            .await?;
        let last_seen_unix_time = db
            .db_read(move |mut cmds| cmds.profile().data().profile_last_seen_time(account_id))
            .await?;
        let automatic_profile_search_last_seen_time = db
            .db_read(move |mut cmds| {
                cmds.profile()
                    .search()
                    .automatic_profile_search_last_seen_time(account_id)
            })
            .await?;
        let automatic_profile_search_settings = db
            .db_read(move |mut cmds| {
                cmds.profile()
                    .search()
                    .automatic_profile_search_settings(account_id)
            })
            .await?;
        let profile_name_moderation_state = db
            .db_read(move |mut cmds| {
                cmds.profile()
                    .moderation()
                    .profile_name_moderation_state(account_id)
            })
            .await?;
        let profile_text_moderation_state = db
            .db_read(move |mut cmds| {
                cmds.profile()
                    .moderation()
                    .profile_text_moderation_state(account_id)
            })
            .await?;

        let mut cache_profile = CacheProfile::new(
            account_id.uuid,
            profile,
            state.into(),
            attributes,
            filters,
            last_seen_unix_time,
            automatic_profile_search_last_seen_time,
            automatic_profile_search_settings,
            profile_name_moderation_state,
            profile_text_moderation_state,
        );

        let location_area = index_writer.coordinates_to_area(
            profile_location,
            cache_profile.state.min_distance_km_filter,
            cache_profile.state.max_distance_km_filter,
        );
        cache_profile.location.current_position = location_area.clone();
        cache_profile.location.current_iterator = index_iterator
            .new_iterator_state(&location_area, cache_profile.state.random_profile_order);

        // Chat

        let cache_chat = CacheChat::default();

        // Setup cache and profile index

        let entry = CacheEntry::new(
            cache_account,
            cache_profile,
            cache_media,
            cache_chat,
            cache_common,
        );
        let location_index_profile_data = entry.location_index_profile_data();
        cache
            .insert_account_if_not_exists(account_id, entry)
            .await
            .with_info(account_id)?;

        cache
            .load_tokens_from_db_and_return_entry(account_id, login_session)
            .await?;

        let account = db
            .db_read(move |mut cmds| cmds.common().account(account_id))
            .await?;
        if account.profile_visibility().is_currently_public() {
            index_writer
                .update_profile_data(
                    account_id.uuid,
                    location_index_profile_data,
                    location_area.profile_location(),
                )
                .await
                .change_context(CacheError::Init)?;
        }

        Ok(())
    }
}

pub struct DbReaderAll<'a> {
    db_reader: DbReaderRaw<'a>,
}

impl<'a> DbReaderAll<'a> {
    fn new(db_reader: DbReaderRaw<'a>) -> Self {
        Self { db_reader }
    }

    pub async fn db_read<
        T: FnOnce(database::DbReadMode<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, CacheError> {
        self.db_reader
            .db_read(cmd)
            .await
            .change_context(CacheError::Init)
    }
}
