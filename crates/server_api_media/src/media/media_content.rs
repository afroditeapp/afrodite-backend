use axum::{Extension, extract::State};
use model_media::{AccountIdInternal, GetMediaContentResult, MyProfileContent};
use server_api::{S, create_open_api_router};
use server_data_media::read::GetReadMediaCommands;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_MEDIA_CONTENT_INFO: &str = "/media_api/media_content_info";

/// Get my profile and security content
#[utoipa::path(
    get,
    path = PATH_GET_MEDIA_CONTENT_INFO,
    responses(
        (status = 200, description = "Successful.", body = GetMediaContentResult),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_media_content_info(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetMediaContentResult>, StatusCode> {
    MEDIA.get_media_content_info.incr();

    let internal = state
        .read()
        .media()
        .current_account_media(account_id)
        .await?;

    let profile_content_version = internal.profile_content_version_uuid;
    let security_content = internal
        .security_content_id
        .as_ref()
        .map(|v| v.clone().into());
    let info: MyProfileContent = internal.into();

    let sync_version = state
        .read()
        .media()
        .media_content_sync_version(account_id)
        .await?;

    let r = GetMediaContentResult {
        profile_content: info,
        profile_content_version,
        security_content,
        sync_version,
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_media_content,
        get_media_content_info,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_MEDIA_CONTENT_COUNTERS_LIST,
    get_media_content_info,
);
