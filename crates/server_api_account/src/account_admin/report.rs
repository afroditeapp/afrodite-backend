use axum::{extract::State, Extension};
use model_account::{
    AccountIdInternal, GetAccountReportList, Permissions, ProcessAccountReport
};
use server_api::{
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple, S,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_ACCOUNT_REPORT_PENDING_PROCESSING_LIST: &str =
    "/account_api/admin/account_report_pending_processing";

#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_REPORT_PENDING_PROCESSING_LIST,
    responses(
        (status = 200, description = "Successful", body = GetAccountReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_report_pending_processing_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetAccountReportList>, StatusCode> {
    ACCOUNT.get_account_report_pending_processing_list.incr();

    if !permissions.admin_process_account_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .account_admin()
        .report()
        .get_report_list()
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_ACCOUNT_REPORT: &str = "/account_api/admin/process_account_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_ACCOUNT_REPORT,
    request_body = ProcessAccountReport,
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
pub async fn post_process_account_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessAccountReport>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_process_account_report.incr();

    if !permissions.admin_process_account_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.account_admin()
            .report()
            .process_report(moderator_id, creator, target, data.content)
            .await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_account_report,
        get_account_report_pending_processing_list,
        post_process_account_report,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_ADMIN_ACCOUNT_REPORT_COUNTERS_LIST,
    get_account_report_pending_processing_list,
    post_process_account_report,
);
