use axum::{extract::State, Extension};
use model_profile::{
    AccountIdInternal, GetProfileNameReportList, GetProfileTextReportList, Permissions, ProcessProfileNameReport, ProcessProfileTextReport
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

const PATH_GET_WAITING_PROFILE_NAME_REPORT_PAGE: &str =
    "/profile_api/admin/waiting_profile_name_report_page";

#[utoipa::path(
    get,
    path = PATH_GET_WAITING_PROFILE_NAME_REPORT_PAGE,
    responses(
        (status = 200, description = "Successful", body = GetProfileNameReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_waiting_profile_name_report_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetProfileNameReportList>, StatusCode> {
    PROFILE.get_waiting_profile_name_report_page.incr();

    if !permissions.admin_process_profile_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .report()
        .get_profile_name_report_list()
        .await?;

    Ok(r.into())
}

const PATH_GET_WAITING_PROFILE_TEXT_REPORT_PAGE: &str =
    "/profile_api/admin/waiting_profile_text_report_page";

#[utoipa::path(
    get,
    path = PATH_GET_WAITING_PROFILE_TEXT_REPORT_PAGE,
    responses(
        (status = 200, description = "Successful", body = GetProfileTextReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_waiting_profile_text_report_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetProfileTextReportList>, StatusCode> {
    PROFILE.get_waiting_profile_text_report_page.incr();

    if !permissions.admin_process_profile_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .report()
        .get_profile_text_report_list()
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_PROFILE_NAME_REPORT: &str = "/profile_api/admin/process_profile_name_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_PROFILE_NAME_REPORT,
    request_body = ProcessProfileNameReport,
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
pub async fn post_process_profile_name_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessProfileNameReport>,
) -> Result<(), StatusCode> {
    PROFILE.post_process_profile_name_report.incr();

    if !permissions.admin_process_profile_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.profile_admin()
            .report()
            .process_profile_name_report(moderator_id, creator, target, data.profile_name)
            .await?;
        Ok(())
    })?;

    Ok(())
}

const PATH_POST_PROCESS_PROFILE_TEXT_REPORT: &str = "/profile_api/admin/process_profile_text_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_PROFILE_TEXT_REPORT,
    request_body = ProcessProfileTextReport,
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
pub async fn post_process_profile_text_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessProfileTextReport>,
) -> Result<(), StatusCode> {
    PROFILE.post_process_profile_text_report.incr();

    if !permissions.admin_process_profile_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.profile_admin()
            .report()
            .process_profile_text_report(moderator_id, creator, target, data.profile_text)
            .await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_profile_report,
        get_waiting_profile_name_report_page,
        get_waiting_profile_text_report_page,
        post_process_profile_name_report,
        post_process_profile_text_report,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_REPORT_COUNTERS_LIST,
    get_waiting_profile_name_report_page,
    get_waiting_profile_text_report_page,
    post_process_profile_name_report,
    post_process_profile_text_report,
);
