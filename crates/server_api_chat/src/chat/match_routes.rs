//! Match related routes

use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, AllMatchesPage, MatchesIteratorSessionId, MatchesPage, ResetMatchesIteratorResult};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::WriteData, db_write};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
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
        (status = 200, description = "Success.", body = AllMatchesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_matches<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<AllMatchesPage>, StatusCode> {
    CHAT.get_matches.incr();

    let page = state.read().chat().all_matches(id).await?;
    Ok(page.into())
}

#[obfuscate_api]
const PATH_POST_RESET_MATCHES_PAGING: &str = "/chat_api/matches/reset";

#[utoipa::path(
    post,
    path = PATH_POST_RESET_MATCHES_PAGING,
    responses(
        (status = 200, description = "Successfull.", body = ResetMatchesIteratorResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_matches_paging<S: WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ResetMatchesIteratorResult>, StatusCode> {
    CHAT.post_reset_matches_paging.incr();
    let iterator_session_id = db_write!(state, move |cmds| {
        cmds.chat().handle_reset_matches_iterator(account_id)
    })?;
    let r = ResetMatchesIteratorResult {
        s: iterator_session_id.into(),
    };

    Ok(r.into())
}

#[obfuscate_api]
const PATH_POST_GET_NEXT_MATCHES_PAGE: &str = "/chat_api/matches_page";

/// Update matches iterator and get next page
/// of matches. If the page is empty there is no more
/// matches available.
#[utoipa::path(
    post,
    path = PATH_POST_GET_NEXT_MATCHES_PAGE,
    request_body(content = MatchesIteratorSessionId),
    responses(
        (status = 200, description = "Success.", body = MatchesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_matches_page<S: WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<MatchesIteratorSessionId>,
) -> Result<Json<MatchesPage>, StatusCode> {
    CHAT.post_get_next_matches_page.incr();

    let data = state
        .concurrent_write_profile_blocking(
            account_id.as_id(),
            move |cmds| {
                cmds.next_matches_iterator_state(account_id, iterator_session_id)
            }
        )
        .await??;

    if let Some(data) = data {
        // Matches iterator session ID was valid
        let profiles = state
            .read()
            .chat()
            .matches_page(account_id, data)
            .await?;
        Ok(MatchesPage {
            p: profiles,
            error_invalid_iterator_session_id: false,
        }.into())
    } else {
        Ok(MatchesPage {
            p: vec![],
            error_invalid_iterator_session_id: true,
        }.into())
    }
}

pub fn match_router<S: StateBase + ReadData + WriteData>(s: S) -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route(PATH_GET_MATCHES_AXUM, get(get_matches::<S>))
        .route(PATH_POST_RESET_MATCHES_PAGING_AXUM, post(post_reset_matches_paging::<S>))
        .route(PATH_POST_GET_NEXT_MATCHES_PAGE_AXUM, post(post_get_next_matches_page::<S>))
        .with_state(s)
}

create_counters!(ChatCounters, CHAT, CHAT_MATCH_COUNTERS_LIST, get_matches, post_reset_matches_paging, post_get_next_matches_page,);
