use axum::{Extension, extract::State};
use model::{
    AccountIdInternal, PendingAppNotificationList, PendingAppNotificationToDelete,
    PushNotificationFlags,
};
use server_data::{
    app::EventManagerProvider, read::GetReadCommandsCommon, write::GetWriteCommandsCommon,
};
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, WriteData},
    create_open_api_router, db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_PENDING_APP_NOTIFICATIONS: &str = "/common_api/pending_app_notifications";

#[utoipa::path(
    get,
    path = PATH_GET_PENDING_APP_NOTIFICATIONS,
    responses(
        (status = 200, description = "Success.", body = PendingAppNotificationList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_app_notifications(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<PendingAppNotificationList>, StatusCode> {
    COMMON.get_pending_app_notifications.incr();

    let notifications = state
        .read()
        .common()
        .notification()
        .pending_app_notifications(id)
        .await?;

    state
        .event_manager()
        .remove_pending_push_notification_flags_from_cache(
            id,
            PushNotificationFlags::PENDING_APP_NOTIFICATION,
        )
        .await;

    Ok(PendingAppNotificationList { notifications }.into())
}

const PATH_POST_DELETE_PENDING_APP_NOTIFICATIONS: &str =
    "/common_api/pending_app_notifications/delete";

#[utoipa::path(
    post,
    path = PATH_POST_DELETE_PENDING_APP_NOTIFICATIONS,
    request_body = Vec<PendingAppNotificationToDelete>,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_delete_pending_app_notifications(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(notifications): Json<Vec<PendingAppNotificationToDelete>>,
) -> Result<(), StatusCode> {
    COMMON.post_delete_pending_app_notifications.incr();

    db_write!(state, move |cmds| {
        cmds.common()
            .notification()
            .delete_pending_app_notifications(id, notifications)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_notification,
    get_pending_app_notifications,
    post_delete_pending_app_notifications,
);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_NOTIFICATION_COUNTERS_LIST,
    get_pending_app_notifications,
    post_delete_pending_app_notifications,
);
