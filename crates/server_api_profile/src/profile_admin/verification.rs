use axum::{
    Extension,
    extract::{Path, State},
};
use model::{
    AccountId, EditVerificationProfileAgeRange, EditVerificationProfileName, EditVerificationValues,
};
use model_profile::{
    Permissions, PostProfileAgeRangeVerifiedValue, PostProfileNameVerifiedValue,
    ProfileAgeRangeVerificationAdminInfo, ProfileNameVerificationAdminInfo,
};
use server_api::{S, app::GetAccounts, create_open_api_router};
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_PROFILE_AGE_RANGE_VERIFICATION_ADMIN_INFO: &str =
    "/profile_api/profile_age_range_verification_admin_info/{aid}";

/// Get profile age range verification values.
///
/// # Access
/// - Permission [model::Permissions::admin_edit_profile_age_range_verified_value]
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_AGE_RANGE_VERIFICATION_ADMIN_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = ProfileAgeRangeVerificationAdminInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_age_range_verification_admin_info(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<ProfileAgeRangeVerificationAdminInfo>, StatusCode> {
    PROFILE.get_profile_age_range_verification_admin_info.incr();

    if !permissions.admin_edit_profile_age_range_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(requested_account_id).await?;
    let profile_state = state.read().profile().profile_state(internal_id).await?;

    let info = ProfileAgeRangeVerificationAdminInfo {
        profile_age_range_verified: profile_state.profile_age_range_verified,
        profile_age_range_verified_manual: profile_state.profile_age_range_verified_manual,
    };

    Ok(info.into())
}

const PATH_POST_PROFILE_AGE_RANGE_VERIFIED_VALUE: &str =
    "/profile_api/profile_age_range_verified_value";

/// Change profile age range verified value.
///
/// Bot account sets automatic value and human admin account sets manual override value.
///
/// # Access
/// - Permission [model::Permissions::admin_edit_profile_age_range_verified_value]
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_AGE_RANGE_VERIFIED_VALUE,
    request_body = PostProfileAgeRangeVerifiedValue,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_age_range_verified_value(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<model_profile::AccountIdInternal>,
    Json(data): Json<PostProfileAgeRangeVerifiedValue>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_age_range_verified_value.incr();

    if !permissions.admin_edit_profile_age_range_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let profile_owner_id = state.get_internal_id(data.account_id).await?;

    state
        .data_all_access()
        .edit_verification_values(
            moderator_id,
            EditVerificationValues {
                profile_owner_id,
                security_content: None,
                profile_age_range: Some(EditVerificationProfileAgeRange {
                    current_profile_age: data.current_profile_age,
                    verified_value: data.value,
                }),
                profile_name: None,
            },
        )
        .await?;

    Ok(())
}

const PATH_GET_PROFILE_NAME_VERIFICATION_ADMIN_INFO: &str =
    "/profile_api/profile_name_verification_admin_info/{aid}";

/// Get profile name verification values.
///
/// # Access
/// - Permission [model::Permissions::admin_edit_profile_name_verified_value]
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_NAME_VERIFICATION_ADMIN_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = ProfileNameVerificationAdminInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_name_verification_admin_info(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<ProfileNameVerificationAdminInfo>, StatusCode> {
    PROFILE.get_profile_name_verification_admin_info.incr();

    if !permissions.admin_edit_profile_name_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(requested_account_id).await?;
    let profile_state = state.read().profile().profile_state(internal_id).await?;

    let info = ProfileNameVerificationAdminInfo {
        profile_name_verified: profile_state.profile_name_verified,
        profile_name_verified_manual: profile_state.profile_name_verified_manual,
    };

    Ok(info.into())
}

const PATH_POST_PROFILE_NAME_VERIFIED_VALUE: &str = "/profile_api/profile_name_verified_value";

/// Change profile name verified value.
///
/// Bot account sets automatic value and human admin account sets manual override value.
///
/// # Access
/// - Permission [model::Permissions::admin_edit_profile_name_verified_value]
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_NAME_VERIFIED_VALUE,
    request_body = PostProfileNameVerifiedValue,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_name_verified_value(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<model_profile::AccountIdInternal>,
    Json(data): Json<PostProfileNameVerifiedValue>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_name_verified_value.incr();

    if !permissions.admin_edit_profile_name_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let profile_owner_id = state.get_internal_id(data.account_id).await?;

    state
        .data_all_access()
        .edit_verification_values(
            moderator_id,
            EditVerificationValues {
                profile_owner_id,
                security_content: None,
                profile_age_range: None,
                profile_name: Some(EditVerificationProfileName {
                    current_profile_name: data.current_profile_name,
                    verified_value: data.value,
                }),
            },
        )
        .await?;

    Ok(())
}

create_open_api_router!(
    fn router_admin_verification,
    get_profile_age_range_verification_admin_info,
    post_profile_age_range_verified_value,
    get_profile_name_verification_admin_info,
    post_profile_name_verified_value,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_VERIFICATION_COUNTERS_LIST,
    get_profile_age_range_verification_admin_info,
    post_profile_age_range_verified_value,
    get_profile_name_verification_admin_info,
    post_profile_name_verified_value,
);
