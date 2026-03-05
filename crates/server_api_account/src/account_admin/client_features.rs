use axum::{Extension, extract::State};
use model::{EventToClientInternal, Permissions};
use model_account::SaveInfoBanners;
use server_api::{S, create_open_api_router, db_write};
use server_data_account::write::{GetWriteCommandsAccount, account_admin::SaveInfoBannersResult};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

const PATH_POST_SAVE_INFO_BANNERS: &str = "/account_api/save_info_banners";

/// Save info banners to dynamic client config.
///
/// Existing banners cannot be removed.
///
/// Don't edit [model:InfoBanner::version] field as server will update that.
///
/// # Access
///
/// Permission [model::Permissions::admin_server_edit_info_banners] is required.
#[utoipa::path(
    post,
    path = PATH_POST_SAVE_INFO_BANNERS,
    request_body = SaveInfoBanners,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_save_info_banners(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(request): Json<SaveInfoBanners>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_save_info_banners.incr();

    if !permissions.admin_server_edit_info_banners {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let result = db_write!(state, move |cmds| {
        let result = cmds
            .account_admin()
            .client_features()
            .save_info_banners(request)
            .await?;

        if result == SaveInfoBannersResult::Updated {
            cmds.events()
                .send_connected_event_to_logged_in_clients(
                    EventToClientInternal::ClientConfigChanged,
                )
                .await;
        }

        Ok(result)
    })?;

    match result {
        SaveInfoBannersResult::Updated | SaveInfoBannersResult::NotModified => Ok(()),
        SaveInfoBannersResult::ErrorCurrentStateChanged => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

create_open_api_router!(
    fn router_admin_client_features,
    post_save_info_banners,
);

create_counters!(
    AccountCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_CLIENT_FEATURES_COUNTERS_LIST,
    post_save_info_banners,
);
