use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, AdminNotificationTypes, EventToClientInternal};
use model_profile::{GetProfileAgeAndName, Permissions, ProfileUpdateInternal, SetProfileName};
use server_api::{
    DataError, S,
    app::{AdminNotificationProvider, GetAccounts, GetConfig},
    create_open_api_router, db_write,
};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    app::{ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_GET_PROFILE_AGE_AND_NAME: &str = "/profile_api/get_profile_age_and_name/{aid}";

/// Get profile age and name
///
/// # Access
/// - Permission [model::Permissions::admin_edit_profile_name]
/// - Permission [model::Permissions::admin_find_account_by_email_address]
/// - Permission [model::Permissions::admin_view_permissions]
/// - Permission [model::Permissions::admin_moderate_media_content]
/// - Permission [model::Permissions::admin_moderate_profile_names]
/// - Permission [model::Permissions::admin_moderate_profile_texts]
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_AGE_AND_NAME,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = GetProfileAgeAndName),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_age_and_name(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<GetProfileAgeAndName>, StatusCode> {
    PROFILE.get_profile_age_and_name.incr();

    let access_allowed = permissions.admin_edit_profile_name
        || permissions.admin_find_account_by_email_address
        || permissions.admin_view_permissions
        || permissions.admin_moderate_media_content
        || permissions.admin_moderate_profile_names
        || permissions.admin_moderate_profile_texts;

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let profile_owner_id = state.get_internal_id(account_id).await?;

    let r = state.read().profile().profile(profile_owner_id).await?;
    let r = GetProfileAgeAndName {
        age: r.profile.age,
        name: r.profile.name,
    };

    Ok(r.into())
}

const PATH_POST_SET_PROFILE_NAME: &str = "/profile_api/set_profile_name";

/// Set profile name
///
/// The new name has the same requirements as in
/// [crate::profile::post_profile] route documentation.
///
/// # Access
/// - Permission [model::Permissions::admin_edit_profile_name]
#[utoipa::path(
    post,
    path = PATH_POST_SET_PROFILE_NAME,
    request_body = SetProfileName,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_profile_name(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(info): Json<SetProfileName>,
) -> Result<(), StatusCode> {
    PROFILE.post_set_profile_name.incr();

    let access_allowed = permissions.admin_edit_profile_name;

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let profile_owner_id = state.get_internal_id(info.account).await?;

    db_write!(state, move |cmds| {
        let profile = cmds
            .read()
            .profile()
            .profile(profile_owner_id)
            .await?
            .profile;

        let profile_update = ProfileUpdateInternal {
            ptext: profile.ptext.clone(),
            name: info.name,
            age: profile.age,
            attributes: profile
                .attributes
                .iter()
                .cloned()
                .map(|v| v.into())
                .collect(),
        };
        let profile_update = profile_update
            .validate(
                cmds.profile_attributes().schema(),
                cmds.config().profile_name_regex(),
                &profile,
                None,
                cmds.read().common().is_bot(profile_owner_id).await?,
            )
            .into_error_string(DataError::NotAllowed)?;
        cmds.profile()
            .profile(profile_owner_id, profile_update)
            .await?;

        cmds.events()
            .send_connected_event(profile_owner_id, EventToClientInternal::ProfileChanged)
            .await?;

        Ok(())
    })?;

    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ModerateProfileNamesBot)
        .await;

    Ok(())
}

create_open_api_router!(
        fn router_admin_profile_data,
        get_profile_age_and_name,
        post_set_profile_name,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_DATA_COUNTERS_LIST,
    get_profile_age_and_name,
    post_set_profile_name,
);
