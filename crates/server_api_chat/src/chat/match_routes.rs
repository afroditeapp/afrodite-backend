//! Match related routes

use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, MatchesPage};
use obfuscate_api_macro::obfuscate_api;
use server_data_chat::read::GetReadChatCommands;
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::{ReadData, StateBase};

#[obfuscate_api]
const PATH_GET_MATCHES: &str = "/chat_api/matches";

/// Get matches
#[utoipa::path(
    get,
    path = PATH_GET_MATCHES,
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

pub fn match_router<S: StateBase + ReadData>(s: S) -> Router {
    use axum::routing::get;

    Router::new()
        .route(PATH_GET_MATCHES_AXUM, get(get_matches::<S>))
        .with_state(s)
}

create_counters!(ChatCounters, CHAT, CHAT_MATCH_COUNTERS_LIST, get_matches,);
