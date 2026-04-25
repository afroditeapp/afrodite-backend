use axum::{Extension, extract::State};
use model::AdminBotNotificationTypes;
use model_media::{AccountIdInternal, ContentId};
use server_api::{S, app::AdminNotificationProvider, create_open_api_router, db_write};
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

create_open_api_router!(
        fn router_security_content,
        put_security_content_info,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_SECURITY_CONTENT_COUNTERS_LIST,
    put_security_content_info,
);
