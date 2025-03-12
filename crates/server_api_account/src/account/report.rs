use axum::{extract::{Query, State}, Extension};
use model::{AccountIdInternal, CustomReportsFileHash, ReportQueryParams};
use model_account::{AccountReport, GetCustomReportsConfigResult, UpdateAccountReport};
use server_api::{app::GetConfig, create_open_api_router, S};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_ACCOUNT_REPORT: &str = "/account_api/account_report";

/// Get account report
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_REPORT,
    params(ReportQueryParams),
    responses(
        (status = 200, description = "Successfull.", body = AccountReport),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Query(report): Query<ReportQueryParams>,
) -> Result<Json<AccountReport>, StatusCode> {
    ACCOUNT.get_account_report.incr();

    let target = state.get_internal_id(report.target).await?;

    let report = state.read().account().report().get_report(
        account_id,
        target,
    ).await?;

    Ok(report.into())
}

const PATH_POST_ACCOUNT_REPORT: &str = "/account_api/account_report";

/// Update account report.
#[utoipa::path(
    post,
    path = PATH_POST_ACCOUNT_REPORT,
    request_body = UpdateAccountReport,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateAccountReport>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_report.incr();

    let target = state.get_internal_id(update.target).await?;

    db_write!(state, move |cmds| cmds
        .account()
        .report()
        .update_report(account_id, target, update.content))?;

    Ok(())
}

const PATH_POST_GET_CUSTOM_CONFIG_REPORTS: &str = "/account_api/custom_reports_config";

#[utoipa::path(
    post,
    path = PATH_POST_GET_CUSTOM_CONFIG_REPORTS,
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
        GetCustomReportsConfigResult {
            config: None,
        }
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_account_report,
        get_account_report,
        post_account_report,
        post_get_custom_reports_config,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REPORT_COUNTERS_LIST,
    get_account_report,
    post_account_report,
    post_get_custom_reports_config,
);
