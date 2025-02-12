use axum::{extract::State, Extension};
use model_profile::{
    AccountIdInternal, GetProfileReportList, Permissions, ProcessProfileReport
};
use server_api::{
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple, S,
};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_PROFILE_REPORT_PENDING_PROCESSING_LIST: &str =
    "/profile_api/admin/profile_report_pending_processing";

#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_REPORT_PENDING_PROCESSING_LIST,
    responses(
        (status = 200, description = "Successful", body = GetProfileReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_report_pending_processing_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetProfileReportList>, StatusCode> {
    PROFILE.get_profile_report_pending_processing_list.incr();

    if !permissions.admin_process_profile_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .report()
        .get_report_list()
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_PROFILE_REPORT: &str = "/profile_api/admin/process_profile_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_PROFILE_REPORT,
    request_body = ProcessProfileReport,
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
pub async fn post_process_profile_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessProfileReport>,
) -> Result<(), StatusCode> {
    PROFILE.post_process_profile_report.incr();

    if !permissions.admin_process_profile_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.profile_admin()
            .report()
            .process_report(moderator_id, creator, target, data.content)
            .await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_profile_report,
        get_profile_report_pending_processing_list,
        post_process_profile_report,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_REPORT_COUNTERS_LIST,
    get_profile_report_pending_processing_list,
    post_process_profile_report,
);
