use axum::{
    body::Body, extract::{Path, State},
};
use axum_extra::TypedHeader;
use headers::{ContentLength, ContentType};
use model::{MapTileX, MapTileY, MapTileZ};
use obfuscate_api_macro::obfuscate_api;
use server_api::create_open_api_router;
use simple_backend::{app::GetTileMap, create_counters};
use tracing::error;
use utoipa_axum::router::OpenApiRouter;

use crate::{app::StateBase, utils::StatusCode};

#[obfuscate_api]
const PATH_GET_MAP_TILE: &str = "/media_api/map_tile/{z}/{x}/{y}";

/// Get map tile PNG file.
///
/// Returns a .png even if the URL does not have it.
#[utoipa::path(
    get,
    path = PATH_GET_MAP_TILE,
    params(MapTileZ, MapTileX, MapTileY),
    responses(
        (status = 200, description = "Get map tile PNG file.", body = inline(model::BinaryData), content_type = "image/png"),
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
) -> Result<(TypedHeader<ContentType>, TypedHeader<ContentLength>, Body), StatusCode> {
    MEDIA.get_map_tile.incr();

    let y_string = y.y.trim_end_matches(".png");
    let y = y_string
        .parse::<u32>()
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let byte_count_and_data_stream = state
        .tile_map()
        .map_tile_byte_count_and_byte_stream(z.z, x.x, y)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match byte_count_and_data_stream {
        Some((byte_count, data_stream)) =>
            Ok((TypedHeader(ContentType::png()), TypedHeader(ContentLength(byte_count)), Body::from_stream(data_stream))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub fn tile_map_router<S: StateBase + GetTileMap>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_map_tile::<S>,
    )
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_TILE_MAP_COUNTERS_LIST,
    get_map_tile,
);
