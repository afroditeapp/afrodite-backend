//! Match related routes

use axum::{Extension, extract::State};
use model_chat::{AccountIdInternal, MatchesIteratorState, MatchesPage};
use server_api::{S, create_open_api_router};
use server_data_chat::read::GetReadChatCommands;
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_INITIAL_MATCHES_ITERATOR_STATE: &str = "/chat_api/matches/initial_state";

#[utoipa::path(
    get,
    path = PATH_GET_INITIAL_MATCHES_ITERATOR_STATE,
    responses(
        (status = 200, description = "Successfull.", body = MatchesIteratorState),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_initial_matches_iterator_state(
    State(state): State<S>,
) -> Result<Json<MatchesIteratorState>, StatusCode> {
    CHAT.get_initial_matches_iterator_state.incr();
    let iterator_state = state
        .read()
        .chat()
        .get_initial_matches_iterator_state()
        .await?;
    Ok(iterator_state.into())
}

const PATH_POST_GET_MATCHES_ITERATOR_PAGE: &str = "/chat_api/matches";

/// Get requested page of matches iterator page.
/// If the page is empty there is no more
/// matches available.
#[utoipa::path(
    post,
    path = PATH_POST_GET_MATCHES_ITERATOR_PAGE,
    request_body(content = MatchesIteratorState),
    responses(
        (status = 200, description = "Success.", body = MatchesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_matches_iterator_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_state): Json<MatchesIteratorState>,
) -> Result<Json<MatchesPage>, StatusCode> {
    CHAT.post_get_matches_iterator_page.incr();
    let profiles = state
        .read()
        .chat()
        .matches_page(account_id, iterator_state)
        .await?;
    Ok(MatchesPage { p: profiles }.into())
}

create_open_api_router!(
        fn router_match,
        get_initial_matches_iterator_state,
        post_get_matches_iterator_page,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_MATCH_COUNTERS_LIST,
    get_initial_matches_iterator_state,
    post_get_matches_iterator_page,
);
