//! Match related routes

use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, MatchesPage};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

pub const PATH_GET_MATCHES: &str = "/chat_api/matches";

/// Get matches
#[utoipa::path(
    get,
    path = "/chat_api/matches",
    responses(
        (status = 200, description = "Success.", body = MatchesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_matches<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<MatchesPage>, StatusCode> {
    CHAT.get_matches.incr();

    let page = state.read().chat().all_matches(id).await?;
    Ok(page.into())
}

pub fn match_router(s: crate::app::S) -> Router {
    use axum::routing::get;

    use crate::app::S;

    Router::new()
        .route(PATH_GET_MATCHES, get(get_matches::<S>))
        .with_state(s)
}

create_counters!(ChatCounters, CHAT, CHAT_MATCH_COUNTERS_LIST, get_matches,);
