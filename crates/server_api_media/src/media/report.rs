use axum::{extract::State, Extension};
use model::UpdateReportResult;
use model_media::{AccountIdInternal, UpdateProfileContentReport};
use server_api::{create_open_api_router, db_write_multiple, S};
use server_data_media::write::GetWriteCommandsMedia;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, WriteData},
    utils::{Json, StatusCode},
};

const PATH_POST_PROFILE_CONTENT_REPORT: &str = "/media_api/profile_content_report";

/// Report profile content.
///
/// If profile content is reported and it is bot moderated, the content's
/// moderation state changes to
/// [model_media::ContentModerationState::WaitingHumanModeration].
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_CONTENT_REPORT,
    request_body = UpdateProfileContentReport,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_content_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateProfileContentReport>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    MEDIA.post_profile_content_report.incr();

    let target = state.get_internal_id(update.target).await?;

    let result = db_write_multiple!(state, move |cmds| cmds
        .media()
        .report()
        .update_report(account_id, target, update.content)
        .await)?;

    Ok(result.into())
}

create_open_api_router!(
        fn router_media_report,
        post_profile_content_report,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_REPORT_COUNTERS_LIST,
    post_profile_content_report,
);
