use axum::{Extension, extract::State};
use model::{ImageProcessingDynamicConfig, Permissions};
use server_api::{S, app::ReadData, create_open_api_router, utils::StatusCode};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::{app::GetSimpleBackendConfig, create_counters, image::ImageProcess};

use crate::{app::WriteData, db_write, utils::Json};

/// Get image processing configuration
///
/// # Permissions
/// Requires admin_server_view_image_processing_config.
#[utoipa::path(
    get,
    path = "/media_api/image_processing_config",
    responses(
        (status = 200, description = "Image processing configuration", body = ImageProcessingDynamicConfig),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("access_token" = []))
)]
pub async fn get_image_processing_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<ImageProcessingDynamicConfig>, StatusCode> {
    if !api_caller_permissions.admin_server_view_image_processing_config {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let config = state.read().media_admin().image_processing_config().await?;

    Ok(Json(config.unwrap_or_default()))
}

/// Update image processing configuration
///
/// # Permissions
/// Requires admin_server_edit_image_processing_config.
#[utoipa::path(
    post,
    path = "/media_api/image_processing_config",
    request_body = ImageProcessingDynamicConfig,
    responses(
        (status = 200, description = "Image processing configuration updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("access_token" = []))
)]
pub async fn post_image_processing_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(config): Json<ImageProcessingDynamicConfig>,
) -> Result<(), StatusCode> {
    if !api_caller_permissions.admin_server_edit_image_processing_config {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let simple_backend_config = state.simple_backend_config();
    let image_process_config = simple_backend_config.image_process_config(config.clone());

    db_write!(state, move |cmds| {
        cmds.media_admin()
            .image_processing_config()
            .upsert_image_processing_config(&config)
            .await?;

        Ok(())
    })?;

    ImageProcess::update_config_if_process_is_running(image_process_config)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

create_open_api_router!(
    fn router_admin_config,
    get_image_processing_config,
    post_image_processing_config,
);

create_counters!(
    MediaAdminConfigCounters,
    MEDIA_ADMIN_CONFIG,
    MEDIA_ADMIN_CONFIG_COUNTERS_LIST,
    get_image_processing_config,
    post_image_processing_config,
);
