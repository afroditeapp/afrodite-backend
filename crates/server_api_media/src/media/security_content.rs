use axum::{Extension, extract::State};
use model::AdminBotNotificationTypes;
use model_media::{
    AccountIdInternal, ContentId, PostSecurityContentVerificationQueueItem,
    PostSecurityContentVerificationQueueItemResult, SecurityContentVerificationQueueStatus,
};
use server_api::{
    S, SecurityContentVerificationQueueAddError,
    app::{
        AdminNotificationProvider, ApiLimitsProvider, GetConfig, ReadData,
        SecurityContentVerificationQueueProvider,
    },
    create_open_api_router, db_write,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

const PATH_PUT_SECURITY_CONTENT_INFO: &str = "/media_api/security_content_info";

/// Set current security content for current account.
///
/// This also moves the content to moderation if it is not already
/// in moderation or moderated.
///
/// # Restrictions
/// - The content must be owned by the account.
/// - The content must be an image.
/// - The content must be captured by client.
/// - The content must have face detected.
#[utoipa::path(
    put,
    path = PATH_PUT_SECURITY_CONTENT_INFO,
    request_body = ContentId,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_security_content_info(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(content_id): Json<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.put_security_content_info.incr();

    let changed = db_write!(state, move |cmds| {
        let content_id = cmds
            .read()
            .media()
            .content_id_internal(api_caller_account_id, content_id)
            .await?;
        cmds.media().update_security_content(content_id).await
    })?;

    if changed {
        state
            .admin_notification()
            .send_bot_notification_if_needed(
                AdminBotNotificationTypes::VERIFY_MEDIA_CONTENT_FACE_BOT,
            )
            .await;
    }

    Ok(())
}

const PATH_GET_SECURITY_CONTENT_VERIFICATION_QUEUE_STATUS: &str =
    "/media_api/security_content_verification_queue";

/// Get security content verification queue status for current account.
#[utoipa::path(
    get,
    path = PATH_GET_SECURITY_CONTENT_VERIFICATION_QUEUE_STATUS,
    responses(
        (status = 200, description = "Successful.", body = SecurityContentVerificationQueueStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_content_verification_queue_status(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<SecurityContentVerificationQueueStatus>, StatusCode> {
    MEDIA.get_security_content_verification_queue_status.incr();

    let queue_position = state
        .security_content_verification_queue()
        .queue_position(api_caller_account_id)
        .await;

    Ok(SecurityContentVerificationQueueStatus { queue_position }.into())
}

const PATH_POST_SECURITY_CONTENT_VERIFICATION_QUEUE_ITEM: &str =
    "/media_api/security_content_verification_queue";

/// Add security content verification request to queue for current account.
///
/// Queue rules:
/// - One account can have only one pending item in queue.
/// - Queue maximum length is configured with media limits.
/// - Provided security content must match current security content.
#[utoipa::path(
    post,
    path = PATH_POST_SECURITY_CONTENT_VERIFICATION_QUEUE_ITEM,
    request_body = PostSecurityContentVerificationQueueItem,
    responses(
        (status = 200, description = "Successful.", body = PostSecurityContentVerificationQueueItemResult),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn post_security_content_verification_queue_item(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<PostSecurityContentVerificationQueueItem>,
) -> Result<Json<PostSecurityContentVerificationQueueItemResult>, StatusCode> {
    MEDIA.post_security_content_verification_queue_item.incr();

    state
        .api_limits(api_caller_account_id)
        .media()
        .post_security_content_verification_queue_item()
        .await?;

    let PostSecurityContentVerificationQueueItem {
        security_content,
        verification_method,
        verification_data,
    } = data;

    let content_id_internal = state
        .read()
        .media()
        .content_id_internal(api_caller_account_id, security_content)
        .await?;

    let current_security_content = state
        .read()
        .media()
        .current_account_media(api_caller_account_id)
        .await?
        .security_content_id
        .map(|v| v.id);

    if current_security_content != Some(*content_id_internal.as_db_id()) {
        return Ok(
            PostSecurityContentVerificationQueueItemResult::error_security_content_not_current()
                .into(),
        );
    }

    let max_queue_length = state
        .config()
        .limits_media()
        .security_content_verification_queue_max_length;

    let add_result = state
        .security_content_verification_queue()
        .add(
            api_caller_account_id,
            security_content,
            verification_method,
            verification_data,
            max_queue_length,
        )
        .await;

    let result = match add_result {
        Ok(()) => PostSecurityContentVerificationQueueItemResult::success(),
        Err(SecurityContentVerificationQueueAddError::AlreadyQueued) => {
            PostSecurityContentVerificationQueueItemResult::error_already_in_queue()
        }
        Err(SecurityContentVerificationQueueAddError::QueueFull) => {
            PostSecurityContentVerificationQueueItemResult::error_queue_full()
        }
    };

    Ok(result.into())
}

create_open_api_router!(
    fn router_security_content,
    put_security_content_info,
    get_security_content_verification_queue_status,
    post_security_content_verification_queue_item,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_SECURITY_CONTENT_COUNTERS_LIST,
    put_security_content_info,
    get_security_content_verification_queue_status,
    post_security_content_verification_queue_item,
);
