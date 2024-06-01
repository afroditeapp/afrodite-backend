use std::collections::hash_map::Entry;

use config::Config;
use database::{CurrentReadHandle, DbReaderRaw, DieselConnection, DieselDatabaseError};
use error_stack::{Result, ResultExt};
use model::AccountIdInternal;
pub use server_common::data::cache::CacheError;
use server_common::data::WithInfo;
use server_data::{
    cache::{CachedProfile, DatabaseCache},
    index::{LocationIndexIteratorHandle, LocationIndexManager, LocationIndexWriteHandle},
};
use tracing::info;

pub struct DbDataToCacheLoader;

impl DbDataToCacheLoader {
    pub async fn load_to_cache(
        cache: &DatabaseCache,
        current_db: &CurrentReadHandle,
        location_index: &LocationIndexManager,
        config: &Config,
    ) -> Result<(), CacheError> {
        // Load data from database to memory.
        info!("Starting to load data from database to memory");

        let db = DbReaderAll::new(DbReaderRaw::new(current_db));
        let accounts = db
            .db_read_account(move |mut cmd| cmd.account().data().account_ids_internal())
            .await
            .change_context(CacheError::Init)?;

        for id in accounts {
            Self::load_account_from_db(
                cache,
                id,
                config,
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
        config: &Config,
        current_db: &CurrentReadHandle,
        index_iterator: LocationIndexIteratorHandle<'_>,
        index_writer: LocationIndexWriteHandle<'_>,
    ) -> Result<(), CacheError> {
        let db = DbReaderAll::new(DbReaderRaw::new(current_db));
        let public_id = db
            .db_read_common(move |mut cmds| cmds.common().state().public_id(account_id))
            .await?;

        cache
            .insert_account_if_not_exists(account_id, public_id)
            .await
            .with_info(account_id)?;

        let read_lock = cache.accounts().read().await;
        let account_entry = read_lock
            .get(&account_id.as_id())
            .ok_or(CacheError::KeyNotExists.report())?;

        let access_token = db
            .db_read_common(move |mut cmds| cmds.common().token().access_token(account_id))
            .await?;
        if let Some(key) = access_token {
            let mut access_tokens = cache.access_tokens().write().await;
            match access_tokens.entry(key) {
                Entry::Vacant(e) => {
                    e.insert(account_entry.clone());
                }
                Entry::Occupied(_) => return Err(CacheError::AlreadyExists.report()),
            }
        }

        let mut entry = account_entry.cache.write().await;

        // Common
        let capabilities = db
            .db_read_common(move |mut cmds| cmds.common().state().account_capabilities(account_id))
            .await?;
        entry.capabilities = capabilities;
        let state = db
            .db_read_common(move |mut cmds| cmds.common().state().account_state_related_shared_state(account_id))
            .await?;
        entry.account_state_related_shared_state = state;

        if config.components().account {
            // empty
        }

        if config.components().profile {
            let profile = db
                .db_read_profile(move |mut cmds| cmds.profile().data().profile_internal(account_id))
                .await?;
            let state = db
                .db_read_profile(move |mut cmds| cmds.profile().data().profile_state(account_id))
                .await?;
            let profile_location = db
                .db_read_profile(move |mut cmds| cmds.profile().data().profile_location(account_id))
                .await?;
            let attributes = db
                .db_read_profile(move |mut cmds| {
                    cmds.profile().data().profile_attribute_values(account_id)
                })
                .await?;
            let filters = db
                .db_read_profile(move |mut cmds| {
                    cmds.profile().data().profile_attribute_filters(account_id)
                })
                .await?;

            let mut profile_data =
                CachedProfile::new(account_id.uuid, profile, state, attributes, filters);

            let location_key = index_writer.coordinates_to_key(&profile_location);
            profile_data.location.current_position = location_key;
            profile_data.location.current_iterator =
                index_iterator.reset_iterator(profile_data.location.current_iterator, location_key);

            let account = db
                .db_read_common(move |mut cmds| cmds.common().account(account_id))
                .await?;
            if account.profile_visibility().is_currently_public() {
                index_writer
                    .update_profile_data(
                        account_id.uuid,
                        profile_data.location_index_profile_data(),
                        location_key,
                    )
                    .await
                    .change_context(CacheError::Init)?;
            }

            entry.profile = Some(Box::new(profile_data));
        }

        if config.components().chat {
            // empty
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

    pub async fn db_read_account<
        T: FnOnce(
                database_account::current::read::CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, CacheError> {
        self.db_reader
            .db_read(|conn| {
                cmd(database_account::current::read::CurrentSyncReadCommands::new(conn))
            })
            .await
            .change_context(CacheError::Init)
    }

    pub async fn db_read_profile<
        T: FnOnce(
                database_profile::current::read::CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, CacheError> {
        self.db_reader
            .db_read(|conn| {
                cmd(database_profile::current::read::CurrentSyncReadCommands::new(conn))
            })
            .await
            .change_context(CacheError::Init)
    }

    pub async fn db_read_common<
        T: FnOnce(
                database::current::read::CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, CacheError> {
        self.db_reader
            .db_read(|conn| cmd(database::current::read::CurrentSyncReadCommands::new(conn)))
            .await
            .change_context(CacheError::Init)
    }
}
