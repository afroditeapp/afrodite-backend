use axum::{Extension, extract::State};
use model::{AccountIdInternal, AdminNotificationTypes, CustomReportsFileHash, UpdateReportResult};
use model_account::{GetCustomReportsConfigResult, UpdateCustomReportBoolean};
use server_api::{
    S,
    app::{AdminNotificationProvider, GetConfig},
    create_open_api_router, db_write_multiple,
};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, WriteData},
    utils::{Json, StatusCode},
};

const PATH_POST_CUSTOM_REPORT_BOOLEAN: &str = "/account_api/custom_report_boolean";

/// Send custom report
#[utoipa::path(
    post,
    path = PATH_POST_CUSTOM_REPORT_BOOLEAN,
    request_body = UpdateCustomReportBoolean,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_custom_report_boolean(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateCustomReportBoolean>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    ACCOUNT.post_custom_report_boolean.incr();

    let target = state.get_internal_id(update.target).await?;

    let r = db_write_multiple!(state, move |cmds| cmds
        .account()
        .report()
        .report_custom_report_boolean(account_id, target, update.custom_report_id, update.value)
        .await)?;

    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ProcessReports)
        .await;

    Ok(r.into())
}

const PATH_POST_GET_CUSTOM_REPORTS_CONFIG: &str = "/account_api/custom_reports_config";

#[utoipa::path(
    post,
    path = PATH_POST_GET_CUSTOM_REPORTS_CONFIG,
    request_body = CustomReportsFileHash,
    responses(
        (status = 200, description = "Successfull.", body = GetCustomReportsConfigResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_custom_reports_config(
    State(state): State<S>,
    Json(requested_hash): Json<CustomReportsFileHash>,
) -> Result<Json<GetCustomReportsConfigResult>, StatusCode> {
    ACCOUNT.post_get_custom_reports_config.incr();

    let r = if Some(requested_hash.hash()) == state.config().custom_reports_sha256() {
        GetCustomReportsConfigResult {
            config: state.config().custom_reports().cloned(),
        }
    } else {
        GetCustomReportsConfigResult { config: None }
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_account_report,
        post_custom_report_boolean,
        post_get_custom_reports_config,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REPORT_COUNTERS_LIST,
    post_custom_report_boolean,
    post_get_custom_reports_config,
);
