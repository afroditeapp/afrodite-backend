


pub struct DbDataToCacheLoader;

impl DbDataToCacheLoader {
    pub async fn load_to_cache(
        current_cache: &DatabaseCache,
        current_db: &CurrentReadHandle,
        location_index: &LocationIndexManager,
        config: &Config,
    ) -> Result<(), CacheError> {
        let cache = Self {
            access_tokens: RwLock::new(HashMap::new()),
            accounts: RwLock::new(HashMap::new()),
        };

        // Load data from database to memory.
        info!("Starting to load data from database to memory");

        let db_reader = DbReader::new(current_db);
        let accounts = db_reader
            .db_read(move |mut cmd| cmd.account().data().account_ids_internal())
            .await
            .change_context(CacheError::Init)?;

        for id in accounts {
            cache
                .load_account_from_db(
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
        Ok(cache)
    }


    pub async fn load_account_from_db(
        &self,
        account_id: AccountIdInternal,
        config: &Config,
        current_db: &CurrentReadHandle,
        index_iterator: LocationIndexIteratorHandle<'_>,
        index_writer: LocationIndexWriteHandle<'_>,
    ) -> Result<(), CacheError> {
        self.insert_account_if_not_exists(account_id)
            .await
            .with_info(account_id)?;

        let read_lock = self.accounts.read().await;
        let account_entry = read_lock
            .get(&account_id.as_id())
            .ok_or(CacheError::KeyNotExists.report())?;

        let access_token = db_read(current_db, move |mut cmds| {
            cmds.account().token().access_token(account_id)
        })
        .await?;

        if let Some(key) = access_token {
            let mut access_tokens = self.access_tokens.write().await;
            match access_tokens.entry(key) {
                Entry::Vacant(e) => {
                    e.insert(account_entry.clone());
                }
                Entry::Occupied(_) => return Err(CacheError::AlreadyExists.report()),
            }
        }

        let mut entry = account_entry.cache.write().await;

        // Common
        let capabilities = db_read(current_db, move |mut cmds| {
            cmds.common().state().account_capabilities(account_id)
        })
        .await?;
        entry.capabilities = capabilities;
        let state = db_read(current_db, move |mut cmds| {
            cmds.common().state().shared_state(account_id)
        })
        .await?;
        entry.shared_state = state;

        if config.components().account {
            // empty
        }

        if config.components().profile {
            let profile = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile_internal(account_id)
            })
            .await?;
            let state = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile_state(account_id)
            })
            .await?;
            let profile_location = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile_location(account_id)
            })
            .await?;
            let attributes = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile_attribute_values(account_id)
            })
            .await?;
            let filters = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile_attribute_filters(account_id)
            })
            .await?;

            let mut profile_data =
                CachedProfile::new(account_id.uuid, profile, state, attributes, filters);

            let location_key = index_writer.coordinates_to_key(&profile_location);
            profile_data.location.current_position = location_key;
            profile_data.location.current_iterator =
                index_iterator.reset_iterator(profile_data.location.current_iterator, location_key);

            let account = db_read(current_db, move |mut cmds| {
                cmds.common().account(account_id)
            })
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
