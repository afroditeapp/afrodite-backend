//! Common routes to all microservices
//!


use axum::{
    extract::{
        State,
    },
};
use model::{
    BackendVersion,
};
use simple_backend::{create_counters};
pub use utils::api::PATH_CONNECT;

use super::utils::{Json};
use crate::{
    app::{BackendVersionProvider},
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
