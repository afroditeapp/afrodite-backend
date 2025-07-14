//! Match related routes

use axum::{Extension, extract::State};
use model_chat::{
    AccountIdInternal, MatchesIteratorSessionId, MatchesPage, ResetMatchesIteratorResult,
};
use server_api::{S, app::WriteData, create_open_api_router, db_write_multiple};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

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
pub async fn post_reset_matches_paging(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ResetMatchesIteratorResult>, StatusCode> {
    CHAT.post_reset_matches_paging.incr();
    let iterator_session_id = db_write_multiple!(state, move |cmds| {
        cmds.chat().handle_reset_matches_iterator(account_id).await
    })?;
    let r = ResetMatchesIteratorResult {
        s: iterator_session_id.into(),
    };

    Ok(r.into())
}

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
pub async fn post_get_next_matches_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<MatchesIteratorSessionId>,
) -> Result<Json<MatchesPage>, StatusCode> {
    CHAT.post_get_next_matches_page.incr();

    let data = state
        .concurrent_write_profile_blocking(account_id.as_id(), move |cmds| {
            cmds.next_matches_iterator_state(account_id, iterator_session_id)
        })
        .await??;

    if let Some(data) = data {
        // Matches iterator session ID was valid
        let profiles = state.read().chat().matches_page(account_id, data).await?;
        Ok(MatchesPage {
            p: profiles,
            error_invalid_iterator_session_id: false,
        }
        .into())
    } else {
        Ok(MatchesPage {
            p: vec![],
            error_invalid_iterator_session_id: true,
        }
        .into())
    }
}

create_open_api_router!(
        fn router_match,
        post_reset_matches_paging,
        post_get_next_matches_page,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_MATCH_COUNTERS_LIST,
    post_reset_matches_paging,
    post_get_next_matches_page,
);
