use axum::{Extension, extract::State};
use model::{
    AccountIdInternal, AdminNotificationTypes, CustomReportsConfigHash, UpdateReportResult,
};
use model_account::{GetCustomReportsConfigResult, UpdateCustomReportEmpty};
use server_api::{
    S,
    app::{AdminNotificationProvider, ApiLimitsProvider, GetConfig},
    create_open_api_router, db_write,
};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, WriteData},
    utils::{Json, StatusCode},
};

const PATH_POST_CUSTOM_REPORT_EMPTY: &str = "/account_api/custom_report_empty";

/// Send custom report without any content
#[utoipa::path(
    post,
    path = PATH_POST_CUSTOM_REPORT_EMPTY,
    request_body = UpdateCustomReportEmpty,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_custom_report_empty(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateCustomReportEmpty>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    ACCOUNT.post_custom_report_empty.incr();
    state.api_limits(account_id).common().send_report().await?;

    let target = state.get_internal_id(update.target).await?;

    let r = db_write!(state, move |cmds| cmds
        .account()
        .report()
        .report_custom_report_empty(account_id, target, update.custom_report_id)
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
    request_body = CustomReportsConfigHash,
    responses(
        (status = 200, description = "Successfull.", body = GetCustomReportsConfigResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_custom_reports_config(
    State(state): State<S>,
    Json(requested_hash): Json<CustomReportsConfigHash>,
) -> Result<Json<GetCustomReportsConfigResult>, StatusCode> {
    ACCOUNT.post_get_custom_reports_config.incr();

    let r = if requested_hash.hash() == state.config().custom_reports_sha256() {
        GetCustomReportsConfigResult {
            config: Some(state.config().custom_reports().clone()),
        }
    } else {
        GetCustomReportsConfigResult { config: None }
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_account_report,
        post_custom_report_empty,
        post_get_custom_reports_config,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REPORT_COUNTERS_LIST,
    post_custom_report_empty,
    post_get_custom_reports_config,
);
