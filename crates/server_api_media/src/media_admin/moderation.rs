use axum::{
    extract::{Query, State},
    Extension,
};
use model::{AdminNotificationTypes, NotificationEvent};
use model_media::{
    AccountIdInternal, EventToClientInternal, GetProfileContentPendingModerationList, GetProfileContentPendingModerationParams, Permissions, PostModerateProfileContent
};
use server_api::{app::{AdminNotificationProvider, GetAccounts, GetConfig}, create_open_api_router, S};
use server_data_media::{read::GetReadMediaCommands, write::{media::InitialContentModerationResult, media_admin::content::ContentModerationMode, GetWriteCommandsMedia}};
use simple_backend::create_counters;
use server_api::app::ReadData;

use crate::{
    app::WriteData,
    db_write_multiple,
    utils::{Json, StatusCode},
};

// TODO(prod): Change moderation related API naming from
//             profile content to media content.

const PATH_GET_PROFILE_CONTENT_PENDING_MODERATION_LIST: &str =
    "/media_api/admin/profile_content_pending_moderation";

/// Get first page of pending profile content moderations. Oldest item is first and count 25.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_CONTENT_PENDING_MODERATION_LIST,
    params(GetProfileContentPendingModerationParams),
    responses(
        (status = 200, description = "Successful", body = GetProfileContentPendingModerationList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_content_pending_moderation_list(
    State(state): State<S>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileContentPendingModerationParams>,
) -> Result<Json<GetProfileContentPendingModerationList>, StatusCode> {
    MEDIA_ADMIN.get_profile_content_pending_moderation_list.incr();

    if !permissions.admin_moderate_media_content {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .media_admin()
        .profile_content_pending_moderation_list_using_moderator_id(moderator_id, params)
        .await?;

    Ok(r.into())
}

const PATH_POST_MODERATE_PROFILE_CONTENT: &str = "/media_api/admin/moderate_profile_content";

/// Rejected category and details can be set only when the content is rejected.
///
/// This route will fail if the content is in slot.
///
/// Also profile visibility moves from pending to normal when
/// all profile content is moderated as accepted.
#[utoipa::path(
    post,
    path = PATH_POST_MODERATE_PROFILE_CONTENT,
    request_body = PostModerateProfileContent,
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
pub async fn post_moderate_profile_content(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<PostModerateProfileContent>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_moderate_profile_content.incr();

    if !permissions.admin_moderate_media_content {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if data.accept && (data.rejected_category.is_some() || data.rejected_details.is_some()) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let content_owner = state.get_internal_id(data.account_id).await?;

    let mode = if data.move_to_human.unwrap_or_default() {
        ContentModerationMode::MoveToHumanModeration
    } else {
        ContentModerationMode::Moderate {
            moderator_id,
            accept: data.accept,
            rejected_category: data.rejected_category,
            rejected_details: data.rejected_details,
        }
    };

    db_write_multiple!(state, move |cmds| {
        let content_id = cmds
            .read()
            .media()
            .content_id_internal(content_owner, data.content_id).await?;
        let info = cmds.media_admin()
            .content()
            .moderate_profile_content(
                mode,
                content_id,
            )
            .await?;

        match info.moderation_result {
            InitialContentModerationResult::AllAccepted { .. } => {
                if cmds.config().components().account {
                    cmds.events()
                        .send_connected_event(
                            content_id.content_owner(),
                            EventToClientInternal::AccountStateChanged,
                        )
                        .await?;
                }
            }
            InitialContentModerationResult::AllModeratedAndNotAccepted |
            InitialContentModerationResult::NoChange => (),
        }

        cmds.events()
            .send_connected_event(
                content_id.content_owner(),
                EventToClientInternal::MediaContentChanged,
            )
            .await?;

        if !data.move_to_human.unwrap_or_default() {
            // Accepted or rejected

            if data.accept {
                cmds.media_admin()
                    .notification()
                    .show_media_content_accepted_notification(
                        content_id.content_owner(),
                    )
                    .await?;
            } else {
                cmds.media_admin()
                    .notification()
                    .show_media_content_rejected_notification(
                        content_id.content_owner(),
                    )
                    .await?;
            }

            cmds.events()
                .send_notification(
                    content_id.content_owner(),
                    NotificationEvent::MediaContentModerationCompleted,
                )
                .await?;
        }

        Ok(())
    })?;

    if data.move_to_human.unwrap_or_default() {
        state
            .admin_notification()
            .send_notification_if_needed(AdminNotificationTypes::ModerateMediaContentHuman)
            .await;
    }

    // TODO(microservice): Add profile visibility change notification
    // to account internal API.

    Ok(())
}

create_open_api_router!(
        fn router_admin_moderation,
        get_profile_content_pending_moderation_list,
        post_moderate_profile_content,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_MODERATION_COUNTERS_LIST,
    get_profile_content_pending_moderation_list,
    post_moderate_profile_content,
);
