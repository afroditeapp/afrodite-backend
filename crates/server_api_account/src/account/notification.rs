use axum::{Extension, extract::State};
use model_account::{AccountAppNotificationSettings, AccountIdInternal};
use server_api::{S, app::WriteData, create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_ACCOUNT_APP_NOTIFICATION_SETTINGS: &str =
    "/account_api/get_account_app_notification_settings";

#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_APP_NOTIFICATION_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = AccountAppNotificationSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_account_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<AccountAppNotificationSettings>, StatusCode> {
    ACCOUNT.get_account_app_notification_settings.incr();

    let settings = state
        .read()
        .account()
        .notification()
        .account_app_notification_settings(id)
        .await?;

    Ok(settings.into())
}

const PATH_POST_ACCOUNT_APP_NOTIFICATION_SETTINGS: &str =
    "/account_api/post_account_app_notification_settings";

#[utoipa::path(
    post,
    path = PATH_POST_ACCOUNT_APP_NOTIFICATION_SETTINGS,
    request_body = AccountAppNotificationSettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_account_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<AccountAppNotificationSettings>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_app_notification_settings.incr();
    db_write!(state, move |cmds| {
        cmds.account()
            .notification()
            .upsert_app_notification_settings(id, settings)
            .await
    })?;
    Ok(())
}

create_open_api_router!(fn router_notification, get_account_app_notification_settings, post_account_app_notification_settings,);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_NOTIFICATION_COUNTERS_LIST,
    get_account_app_notification_settings,
    post_account_app_notification_settings,
);
