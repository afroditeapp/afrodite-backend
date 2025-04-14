use axum::{
    extract::{Path, Query, State},
    Extension,
};
use model::{EventToClientInternal, InitialContentModerationCompletedResult, NotificationEvent, PendingNotificationFlags};
use model_media::{
    AccountId, AccountIdInternal, AccountState,
    GetProfileContentQueryParams, GetProfileContentResult,
    Permissions, ProfileContent, SetProfileContent,
};
use server_api::{app::{ApiUsageTrackerProvider, EventManagerProvider, GetConfig}, create_open_api_router, db_write_multiple, S};
use server_data::read::GetReadCommandsCommon;
use server_data_media::{read::GetReadMediaCommands, write::{media::InitialContentModerationResult, GetWriteCommandsMedia}};
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
    state.api_usage_tracker().incr(account_id, |u| &u.get_profile_content_info).await;

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

    db_write_multiple!(state, move |cmds| {
        let info = cmds
            .media()
            .update_profile_content(api_caller_account_id, new).await?;

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
                cmds.events()
                    .send_notification(
                        api_caller_account_id,
                        Some(NotificationEvent::InitialContentModerationCompleted),
                    )
                    .await?;
            }
            InitialContentModerationResult::AllModeratedAndNotAccepted => {
                cmds.events()
                    .send_notification(
                        api_caller_account_id,
                        Some(NotificationEvent::InitialContentModerationCompleted),
                    )
                    .await?;
            }
            InitialContentModerationResult::NoChange => (),
        }

        Ok(())
    })?;

    // TODO(microservice): Add profile visibility change notification
    // to account internal API.

    Ok(())
}

const PATH_POST_GET_INITIAL_CONTENT_MODERATION_COMPLETED_RESULT: &str = "/media_api/initial_content_moderation_completed_result";

/// Get initial content moderation completed result.
///
#[utoipa::path(
    post,
    path = PATH_POST_GET_INITIAL_CONTENT_MODERATION_COMPLETED_RESULT,
    responses(
        (status = 200, description = "Successfull.", body = InitialContentModerationCompletedResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_initial_content_moderation_completed(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<InitialContentModerationCompletedResult>, StatusCode> {
    MEDIA.post_get_initial_content_moderation_completed.incr();

    let accepted = state.read().media().profile_content_moderated_as_accepted(account_id).await?;

    let request = InitialContentModerationCompletedResult { accepted };

    state
        .event_manager()
        .remove_specific_pending_notification_flags_from_cache(
            account_id,
            PendingNotificationFlags::INITIAL_CONTENT_MODERATION_COMPLETED,
        )
        .await;

    Ok(request.into())
}

create_open_api_router!(
        fn router_profile_content,
        get_profile_content_info,
        put_profile_content,
        post_get_initial_content_moderation_completed,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_PROFILE_CONTENT_COUNTERS_LIST,
    get_profile_content_info,
    put_profile_content,
    post_get_initial_content_moderation_completed,
);
