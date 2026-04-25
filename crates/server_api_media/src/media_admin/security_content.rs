use axum::{
    Extension,
    extract::{Path, State},
};
use model_media::{AccountId, Permissions, SecurityContentAdminInfo};
use server_api::{
    S,
    app::{GetAccounts, ReadData},
    create_open_api_router,
};
use server_data_media::read::GetReadMediaCommands;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_SECURITY_CONTENT_INFO: &str = "/media_api/security_content_info/{aid}";

/// Get current security content for selected profile.
///
/// # Access
///
/// - Permission [model::Permissions::admin_moderate_media_content]
/// - Permission [model::Permissions::admin_edit_media_content_face_verified_value]
/// - Permission [model::Permissions::admin_edit_security_content_verified_value]
#[utoipa::path(
    get,
    path = PATH_GET_SECURITY_CONTENT_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = SecurityContentAdminInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_content_info(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<SecurityContentAdminInfo>, StatusCode> {
    MEDIA_ADMIN.get_security_content_info.incr();

    let access_allowed = permissions.admin_moderate_media_content
        || permissions.admin_edit_media_content_face_verified_value
        || permissions.admin_edit_security_content_verified_value;

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(requested_account_id).await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: SecurityContentAdminInfo = SecurityContentAdminInfo::new(internal_current_media);
    Ok(info.into())
}

create_open_api_router!(
    fn router_admin_security_content,
    get_security_content_info,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_SECURITY_CONTENT_COUNTERS_LIST,
    get_security_content_info,
);
