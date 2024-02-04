use axum::{
    extract::{Path, State},
    Router,
};
use axum_extra::TypedHeader;
use headers::ContentType;
use model::{MapTileX, MapTileY, MapTileZ};
use simple_backend::{app::GetTileMap, create_counters};
use tracing::error;

use crate::api::utils::StatusCode;

pub const PATH_GET_MAP_TILE: &str = "/media_api/map_tile/:z/:x/:y";

/// Get map tile PNG file.
///
/// Returns a .png even if the URL does not have it.
#[utoipa::path(
    get,
    path = "/media_api/map_tile/{z}/{x}/{y}",
    params(MapTileZ, MapTileX, MapTileY),
    responses(
        (status = 200, description = "Get map tile PNG file.", body = Vec<u8>, content_type = "image/png"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_map_tile<S: GetTileMap>(
    State(state): State<S>,
    Path(z): Path<MapTileZ>,
    Path(x): Path<MapTileX>,
    Path(y): Path<MapTileY>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    MEDIA.get_map_tile.incr();

    let y_string = y.y.trim_end_matches(".png");
    let y = y_string
        .parse::<u32>()
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let data = state
        .tile_map()
        .load_map_tile(z.z, x.x, y)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match data {
        Some(data) => Ok((TypedHeader(ContentType::png()), data)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub fn tile_map_router(s: crate::app::S) -> Router {
    use axum::routing::get;

    use crate::app::S;

    Router::new()
        .route(PATH_GET_MAP_TILE, get(get_map_tile::<S>))
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_TILE_MAP_COUNTERS_LIST,
    get_map_tile,
);
