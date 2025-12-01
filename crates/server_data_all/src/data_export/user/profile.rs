use database::{DbReadMode, DieselDatabaseError};
use database_profile::current::read::GetDbReadCommandsProfile;
use model::{ProfileStringModerationCompletedNotification, UnixTime};
use model_chat::{
    AutomaticProfileSearchLastSeenUnixTime, AutomaticProfileSearchSettings, LastSeenUnixTime,
    Location, ProfileAppNotificationSettings, ProfileEditedTime,
};
use model_profile::{
    GetMyProfileResult, GetProfileFilters, InitialProfileAge, ProfileStringModerationCreated,
    SearchAgeRange, SearchGroups,
};
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct UserDataExportJsonProfile {
    profile_filters: GetProfileFilters,
    search_groups: SearchGroups,
    location: Location,
    initial_profile_age: Option<InitialProfileAge>,
    profile_edited_unix_time: ProfileEditedTime,
    search_age_range: SearchAgeRange,
    last_seen_unix_time: LastSeenUnixTime,
    my_profile: GetMyProfileResult,
    favorite_added_time_list: Vec<UnixTime>,
    profile_name_allowlist: Vec<String>,
    profile_app_notification_settings: ProfileAppNotificationSettings,
    profile_string_moderation_completed: ProfileStringModerationCompletedNotification,
    automatic_profile_search_settings: AutomaticProfileSearchSettings,
    automatic_profile_search_last_seen_time: Option<AutomaticProfileSearchLastSeenUnixTime>,
    profile_string_moderation_created: ProfileStringModerationCreated,
}

impl UserDataExportJsonProfile {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let profile_state = current.profile().data().profile_state(id)?;
        let data = Self {
            profile_filters: current.profile().data().profile_filters(id)?,
            search_groups: profile_state.search_group_flags.into(),
            location: current.profile().data().profile_location(id)?,
            initial_profile_age: current.profile().data().initial_profile_age(id)?,
            profile_edited_unix_time: profile_state.profile_edited_unix_time,
            search_age_range: profile_state.into(),
            last_seen_unix_time: current.profile().data().profile_last_seen_time(id)?,
            my_profile: current.profile().data().my_profile(id)?,
            favorite_added_time_list: current.profile().favorite().favorite_added_time_list(id)?,
            profile_name_allowlist: current
                .profile()
                .moderation()
                .my_data_on_database_profile_name_allowlist(id)?,
            profile_app_notification_settings: current
                .profile()
                .notification()
                .app_notification_settings(id)?,
            profile_string_moderation_completed: current
                .profile()
                .notification()
                .profile_string_moderation_completed(id)?,
            automatic_profile_search_settings: current
                .profile()
                .search()
                .automatic_profile_search_settings(id)?,
            automatic_profile_search_last_seen_time: current
                .profile()
                .search()
                .automatic_profile_search_last_seen_time(id)?,
            profile_string_moderation_created: current
                .profile()
                .moderation()
                .profile_string_moderation_created(id)?,
        };
        Ok(data)
    }
}
