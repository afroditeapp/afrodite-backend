use axum::{
    Extension,
    extract::{Path, Query, State},
};
use model::{AccountId, AdminNotificationTypes, NotificationEvent};
use model_profile::{
    AccountIdInternal, EventToClientInternal, GetProfileStringPendingModerationList,
    GetProfileStringPendingModerationParams, GetProfileStringState, GetProfileStringStateParams,
    Permissions, PostModerateProfileString, ProfileStringModerationContentType,
};
use server_api::{
    S,
    app::{AdminNotificationProvider, GetAccounts, WriteData},
    create_open_api_router, db_write,
};
use server_data_profile::{
    read::GetReadProfileCommands,
    write::{GetWriteCommandsProfile, profile_admin::moderation::ModerateProfileValueMode},
};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_PROFILE_STRING_PENDING_MODERATION_LIST: &str =
    "/profile_api/profile_string_pending_moderation";

/// Get first page of pending profile string moderations. Oldest item is first and count 25.
///
/// # Access
/// * [Permissions::admin_moderate_profile_names] or
///   [Permissions::admin_moderate_profile_texts] depending
///   on [GetProfilePendingModerationParams::content_type].
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_STRING_PENDING_MODERATION_LIST,
    params(GetProfileStringPendingModerationParams),
    responses(
        (status = 200, description = "Successful", body = GetProfileStringPendingModerationList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_string_pending_moderation_list(
    State(state): State<S>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStringPendingModerationParams>,
) -> Result<Json<GetProfileStringPendingModerationList>, StatusCode> {
    PROFILE.get_profile_string_pending_moderation_list.incr();

    match params.content_type {
        ProfileStringModerationContentType::ProfileName => {
            if !permissions.admin_moderate_profile_names {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        ProfileStringModerationContentType::ProfileText => {
            if !permissions.admin_moderate_profile_texts {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let r = state
        .read()
        .profile_admin()
        .moderation()
        .profile_string_pending_moderation_list_using_moderator_id(moderator_id, params)
        .await?;

    Ok(r.into())
}

const PATH_POST_MODERATE_PROFILE_STRING: &str = "/profile_api/moderate_profile_string";

/// Rejected category and details can be set only when
/// [PostModerateProfileString::value] is rejected.
///
/// This route will fail if the users's profile name/text is empty or it is not
/// the same name/text that was moderated.
///
/// # Access
/// * [Permissions::admin_moderate_profile_names] or
///   [Permissions::admin_moderate_profile_texts] depending
///   on [PostModerateProfileString::content_type].
#[utoipa::path(
    post,
    path = PATH_POST_MODERATE_PROFILE_STRING,
    request_body = PostModerateProfileString,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_moderate_profile_string(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<PostModerateProfileString>,
) -> Result<(), StatusCode> {
    PROFILE.post_moderate_profile_string.incr();

    match data.content_type {
        ProfileStringModerationContentType::ProfileName => {
            if !permissions.admin_moderate_profile_names {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        ProfileStringModerationContentType::ProfileText => {
            if !permissions.admin_moderate_profile_texts {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    if data.accept && (data.rejected_category.is_some() || data.rejected_details.is_some()) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let string_owner_id = state.get_internal_id(data.id).await?;

    let mode = if data.move_to_human.unwrap_or_default() {
        ModerateProfileValueMode::MoveToHumanModeration
    } else {
        ModerateProfileValueMode::Moderate {
            moderator_id,
            accept: data.accept,
            rejected_category: data.rejected_category,
            rejected_details: data.rejected_details,
        }
    };

    db_write!(state, move |cmds| {
        cmds.profile_admin()
            .moderation()
            .moderate_profile_string(data.content_type, mode, string_owner_id, data.value)
            .await?;

        cmds.events()
            .send_connected_event(string_owner_id, EventToClientInternal::ProfileChanged)
            .await?;

        if !data.move_to_human.unwrap_or_default() {
            // Accepted or rejected

            match data.content_type {
                ProfileStringModerationContentType::ProfileName => {
                    cmds.profile_admin()
                        .notification()
                        .show_profile_name_moderation_completed_notification(
                            string_owner_id,
                            data.accept,
                        )
                        .await?;
                }
                ProfileStringModerationContentType::ProfileText => {
                    cmds.profile_admin()
                        .notification()
                        .show_profile_text_moderation_completed_notification(
                            string_owner_id,
                            data.accept,
                        )
                        .await?;
                }
            }

            cmds.events()
                .send_notification(
                    string_owner_id,
                    NotificationEvent::ProfileStringModerationCompleted,
                )
                .await?;
        }

        Ok(())
    })?;

    if data.move_to_human.unwrap_or_default() {
        state
            .admin_notification()
            .send_notification_if_needed(AdminNotificationTypes::ModerateProfileTextsHuman)
            .await;
    }

    Ok(())
}

const PATH_GET_PROFILE_STRING_STATE: &str = "/profile_api/get_profile_string_state/{aid}";

/// Get profile string state
///
/// # Access
/// * [Permissions::admin_moderate_profile_names] or
///   [Permissions::admin_moderate_profile_texts] depending
///   on [GetProfileStringStateParams::content_type].
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_STRING_STATE,
    params(GetProfileStringStateParams, AccountId),
    responses(
        (status = 200, description = "Successful.", body = GetProfileStringState),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_string_state(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStringStateParams>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<GetProfileStringState>, StatusCode> {
    PROFILE.get_profile_string_state.incr();

    match params.content_type {
        ProfileStringModerationContentType::ProfileName => {
            if !permissions.admin_moderate_profile_names {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        ProfileStringModerationContentType::ProfileText => {
            if !permissions.admin_moderate_profile_texts {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let string_owner_id = state.get_internal_id(account_id).await?;

    let r = state.read().profile().my_profile(string_owner_id).await?;
    let r = match params.content_type {
        ProfileStringModerationContentType::ProfileName => GetProfileStringState {
            value: r.p.name,
            moderation_info: r.name_moderation_info,
        },
        ProfileStringModerationContentType::ProfileText => GetProfileStringState {
            value: r.p.ptext,
            moderation_info: r.text_moderation_info,
        },
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_admin_moderation,
        get_profile_string_pending_moderation_list,
        post_moderate_profile_string,
        get_profile_string_state,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_MODERATION_COUNTERS_LIST,
    get_profile_string_pending_moderation_list,
    post_moderate_profile_string,
    get_profile_string_state,
);
