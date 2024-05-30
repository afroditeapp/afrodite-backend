//! Common routes to all microservices
//!

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use model::{
    AccessToken, AccountIdInternal, AuthPair, BackendVersion, RefreshToken,
    SyncDataVersionFromClient,
};
use simple_backend::{create_counters, web_socket::WebSocketManager};
use simple_backend_utils::IntoReportFromString;
use tracing::{error, info};
pub use utils::api::PATH_CONNECT;

use super::utils::{AccessTokenHeader, Json, StatusCode};
use crate::{
    app::{BackendVersionProvider, GetAccessTokens, GetConfig, ReadData, WriteData},
    result::{Result, WrappedContextExt, WrappedResultExt},
};

pub const PATH_GET_VERSION: &str = "/common_api/version";

/// Get backend version.
#[utoipa::path(
    get,
    path = "/common_api/version",
    security(),
    responses(
        (status = 200, description = "Version information.", body = BackendVersion),
    )
)]
pub async fn get_version<S: BackendVersionProvider>(
    State(state): State<S>,
) -> Json<BackendVersion> {
    COMMON.get_version.incr();
    state.backend_version().into()
}

// TODO(prod): Check access and refresh key lenghts.

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_COUNTERS_LIST,
    get_version,
);
