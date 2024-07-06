use model::{
    AccountIdInternal, Location, Profile, ProfileAndProfileVersion, ProfileAttributeFilterList, ProfileInternal, ProfileStateInternal, UnixTime
};
use server_data::{
    define_server_data_read_commands,
    read::ReadCommandsProvider,
    result::{Result, WrappedContextExt},
    DataError, IntoDataError,
};

define_server_data_read_commands!(ReadCommandsProfile);
define_db_read_command!(ReadCommandsProfile);

impl<C: ReadCommandsProvider> ReadCommandsProfile<C> {
    pub async fn profile_internal(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DataError> {
        self.read_cache(id, move |cache| {
            cache.profile.as_ref().map(|data| data.data.clone())
        })
        .await?
        .ok_or(DataError::NotFound.report())
    }

    pub async fn profile(&self, id: AccountIdInternal) -> Result<ProfileAndProfileVersion, DataError> {
        self.read_cache(id, move |cache| {
            cache
                .profile
                .as_ref()
                .map(|data|
                    ProfileAndProfileVersion {
                        profile: Profile::new(
                            data.data.clone(),
                            data.attributes.attributes().clone(),
                            cache.other_shared_state.unlimited_likes,
                        ),
                        version: data.data.version_uuid,
                        last_seen_time: cache.last_seen_time(),
                    }
                )
        })
        .await?
        .ok_or(DataError::NotFound.report())
    }

    pub async fn profile_location(&self, id: AccountIdInternal) -> Result<Location, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile_location(id))
            .await
            .into_error()
    }

    pub async fn favorite_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.profile().favorite().favorites(id))
            .await
            .into_error()
    }

    pub async fn profile_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileStateInternal, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile_state(id))
            .await
            .into_error()
    }

    pub async fn profile_attribute_filters(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileAttributeFilterList, DataError> {
        self.db_read(move |mut cmds| {
            let filters = cmds.profile().data().profile_attribute_filters(id)?;
            let state = cmds.profile().data().profile_state(id)?;
            Ok(ProfileAttributeFilterList {
                filters,
                last_seen_time_filter: state.last_seen_time_filter,
                unlimited_likes_filter: state.unlimited_likes_filter,
            })
        })
        .await
        .into_error()
    }

    pub async fn benchmark_read_profile_directly_from_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<Profile, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile(id))
            .await
            .into_error()
    }

    pub async fn last_seen_unix_time_in_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile_last_seen_time(id))
            .await
            .into_error()
    }
}
