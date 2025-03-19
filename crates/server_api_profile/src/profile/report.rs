use axum::{extract::State, Extension};
use model::UpdateReportResult;
use model_profile::{AccountIdInternal, UpdateProfileNameReport, UpdateProfileTextReport};
use server_api::{create_open_api_router, db_write_multiple, S};
use server_data_profile::write::GetWriteCommandsProfile;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, WriteData},
    utils::{Json, StatusCode},
};

// TODO(prod): Remove unused report APIs
// TODO(prod): Add bot moderation support to profile name?

const PATH_POST_REPORT_PROFILE_NAME: &str = "/profile_api/report_profile_name";

/// Report profile name
#[utoipa::path(
    post,
    path = PATH_POST_REPORT_PROFILE_NAME,
    request_body = UpdateProfileNameReport,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_report_profile_name(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateProfileNameReport>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    PROFILE.post_report_profile_name.incr();

    let target = state.get_internal_id(update.target).await?;

    let result = db_write_multiple!(state, move |cmds| cmds
        .profile()
        .report()
        .report_profile_name(account_id, target, update.profile_name).await)?;

    Ok(result.into())
}

const PATH_POST_REPORT_PROFILE_TEXT: &str = "/profile_api/report_profile_text";

/// Report profile text
///
/// If profile text is reported and it is bot moderated, the text's
/// moderation state changes to
/// [model_profile::ProfileTextModerationState::WaitingHumanModeration].
#[utoipa::path(
    post,
    path = PATH_POST_REPORT_PROFILE_TEXT,
    request_body = UpdateProfileTextReport,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_report_profile_text(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateProfileTextReport>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    PROFILE.post_report_profile_text.incr();

    let target = state.get_internal_id(update.target).await?;

    let result = db_write_multiple!(state, move |cmds| cmds
        .profile()
        .report()
        .report_profile_text(account_id, target, update.profile_text).await)?;

    Ok(result.into())
}

create_open_api_router!(
        fn router_profile_report,
        post_report_profile_name,
        post_report_profile_text,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_REPORT_COUNTERS_LIST,
    post_report_profile_name,
    post_report_profile_text,
);
