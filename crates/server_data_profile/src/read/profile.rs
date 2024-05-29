use model::{
    AccountIdInternal, Location, Profile, ProfileAttributeFilterList, ProfileInternal,
    ProfileStateInternal,
};
use server_data::define_server_data_read_commands;

use server_data::read::ReadCommandsProvider;
use server_data::{
    result::{Result, WrappedContextExt},
    DataError, IntoDataError,
};

define_server_data_read_commands!(ReadCommandsProfile);
define_db_read_command!(ReadCommandsProfile);

impl <C: ReadCommandsProvider> ReadCommandsProfile<C> {
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

    pub async fn profile(&self, id: AccountIdInternal) -> Result<Profile, DataError> {
        self.read_cache(id, move |cache| {
            cache
                .profile
                .as_ref()
                .map(|data| Profile::new(data.data.clone(), data.attributes.attributes().clone()))
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
            Ok(ProfileAttributeFilterList { filters })
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
}
