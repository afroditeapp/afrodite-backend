use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{
    AcceptedProfileAges, AccountIdInternal, GetMyProfileResult, GetProfileFilters,
    LastSeenUnixTime, Location, Profile, ProfileAndProfileVersion, ProfileInternal,
    ProfileStateInternal,
};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

use crate::cache::CacheReadProfile;

mod notification;
mod search;
mod statistics;

define_cmd_wrapper_read!(ReadCommandsProfile);

impl<'a> ReadCommandsProfile<'a> {
    pub fn statistics(self) -> statistics::ReadCommandsProfileStatistics<'a> {
        statistics::ReadCommandsProfileStatistics::new(self.0)
    }
    pub fn notification(self) -> notification::ReadCommandsProfileNotification<'a> {
        notification::ReadCommandsProfileNotification::new(self.0)
    }
    pub fn search(self) -> search::ReadCommandsProfileSearch<'a> {
        search::ReadCommandsProfileSearch::new(self.0)
    }
}

impl ReadCommandsProfile<'_> {
    pub async fn profile_internal(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DataError> {
        self.read_cache_profile_and_common(id, move |p, _| Ok(p.profile_internal().clone()))
            .await
            .into_error()
    }

    pub async fn profile(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileAndProfileVersion, DataError> {
        self.read_cache_profile_and_common(id, move |data, c| {
            Ok(ProfileAndProfileVersion {
                profile: Profile::new(
                    data.profile_internal().clone(),
                    data.profile_name_moderation_state(),
                    data.profile_text_moderation_state(),
                    data.attributes.attributes().clone(),
                    c.other_shared_state.unlimited_likes,
                ),
                version: data.profile_internal().version_uuid,
                last_seen_time: data.last_seen_time().last_seen_time(),
            })
        })
        .await
        .into_error()
    }

    pub async fn my_profile(&self, id: AccountIdInternal) -> Result<GetMyProfileResult, DataError> {
        let last_seen_time = self
            .read_cache_profile_and_common(id, move |cache, _| {
                Ok(cache.last_seen_time().last_seen_time())
            })
            .await?;

        self.db_read(move |mut cmds| cmds.profile().data().my_profile(id, Some(last_seen_time)))
            .await
            .into_error()
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

    pub async fn profile_filters(
        &self,
        id: AccountIdInternal,
    ) -> Result<GetProfileFilters, DataError> {
        self.db_read(move |mut cmds| {
            let filters = cmds.profile().data().profile_attribute_filters(id)?;
            let state = cmds.profile().data().profile_state(id)?;
            Ok(GetProfileFilters {
                attribute_filters: filters,
                last_seen_time_filter: state.last_seen_time_filter,
                unlimited_likes_filter: state.unlimited_likes_filter,
                min_distance_km_filter: state.min_distance_km_filter,
                max_distance_km_filter: state.max_distance_km_filter,
                profile_created_filter: state.profile_created_time_filter,
                profile_edited_filter: state.profile_edited_time_filter,
                profile_text_min_characters_filter: state.profile_text_min_characters_filter,
                profile_text_max_characters_filter: state.profile_text_max_characters_filter,
                random_profile_order: state.random_profile_order,
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
    ) -> Result<LastSeenUnixTime, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile_last_seen_time(id))
            .await
            .into_error()
    }

    pub async fn accepted_profile_ages(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<AcceptedProfileAges>, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().accepted_profile_ages(id))
            .await
            .into_error()
    }
}
