use axum::{
    Extension,
    extract::{Path, Query, State},
};
use model::{AdminNotificationTypes, EventToClientInternal};
use model_media::{
    AccountId, AccountIdInternal, AccountState, GetProfileContentQueryParams,
    GetProfileContentResult, Permissions, ProfileContent, SetProfileContent,
};
use server_api::{
    S,
    app::{AdminNotificationProvider, ApiUsageTrackerProvider, GetConfig},
    create_open_api_router, db_write,
};
use server_data::read::GetReadCommandsCommon;
use server_data_media::{
    read::GetReadMediaCommands,
    write::{GetWriteCommandsMedia, media::InitialContentModerationResult},
};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_GET_PROFILE_CONTENT_INFO: &str = "/media_api/profile_content_info/{aid}";

/// Get current profile content for selected profile.
///
/// # Access
///
/// ## Own profile
/// Unrestricted access.
///
/// ## Other profiles
/// Normal account state required.
///
/// ## Private other profiles
/// If the profile is a match, then the profile can be accessed if query
/// parameter `is_match` is set to `true`.
///
/// If the profile is not a match, then permission `admin_view_all_profiles`
/// is required.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_CONTENT_INFO,
    params(AccountId, GetProfileContentQueryParams),
    responses(
        (status = 200, description = "Get profile content info.", body = GetProfileContentResult),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_content_info(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(permissions): Extension<Permissions>,
    Path(requested_profile): Path<AccountId>,
    Query(params): Query<GetProfileContentQueryParams>,
) -> Result<Json<GetProfileContentResult>, StatusCode> {
    MEDIA.get_profile_content_info.incr();
    state
        .api_usage_tracker()
        .incr(account_id, |u| &u.get_profile_content_info)
        .await;

    let requested_profile = state.get_internal_id(requested_profile).await?;

    let read_profile_action = || async {
        let internal = state
            .read()
            .media()
            .current_account_media(requested_profile)
            .await?;

        let info: ProfileContent = internal.clone().into();

        match params.version() {
            Some(param_version) if param_version == internal.profile_content_version_uuid => {
                Ok(GetProfileContentResult::current_version_latest_response(
                    internal.profile_content_version_uuid,
                )
                .into())
            }
            _ => Ok(GetProfileContentResult::content_with_version(
                info,
                internal.profile_content_version_uuid,
            )
            .into()),
        }
    };

    if account_id.as_id() == requested_profile.as_id() {
        return read_profile_action().await;
    }

    if account_state != AccountState::Normal {
        return Ok(GetProfileContentResult::empty().into());
    }

    let visibility = state
        .read()
        .common()
        .account(requested_profile)
        .await?
        .profile_visibility()
        .is_currently_public();

    if visibility
        || permissions.admin_view_all_profiles
        || (params.allow_get_content_if_match()
            && state
                .data_all_access()
                .is_match(account_id, requested_profile)
                .await?)
    {
        read_profile_action().await
    } else {
        Ok(GetProfileContentResult::empty().into())
    }
}

const PATH_PUT_PROFILE_CONTENT: &str = "/media_api/profile_content";

/// Set new profile content for current account.
///
/// This also moves the content to moderation if it is not already
/// in moderation or moderated.
///
/// Also profile visibility moves from pending to normal when
/// all profile content is moderated as accepted.
///
/// # Restrictions
/// - All content must be owned by the account.
/// - All content must be images.
/// - First content must have face detected.
#[utoipa::path(
    put,
    path = PATH_PUT_PROFILE_CONTENT,
    request_body(content = SetProfileContent),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_profile_content(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new): Json<SetProfileContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_profile_content.incr();

    db_write!(state, move |cmds| {
        let info = cmds
            .media()
            .update_profile_content(api_caller_account_id, new)
            .await?;

        match info {
            InitialContentModerationResult::AllAccepted { .. } => {
                if cmds.config().components().account {
                    cmds.events()
                        .send_connected_event(
                            api_caller_account_id,
                            EventToClientInternal::AccountStateChanged,
                        )
                        .await?;
                }
            }
            InitialContentModerationResult::AllModeratedAndNotAccepted
            | InitialContentModerationResult::NoChange => (),
        }

        Ok(())
    })?;

    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ModerateInitialMediaContentBot)
        .await;
    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ModerateMediaContentBot)
        .await;

    // TODO(microservice): Add profile visibility change notification
    // to account internal API.

    Ok(())
}

create_open_api_router!(
        fn router_profile_content,
        get_profile_content_info,
        put_profile_content,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_PROFILE_CONTENT_COUNTERS_LIST,
    get_profile_content_info,
    put_profile_content,
);
