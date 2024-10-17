
use axum::{extract::State, Extension};
use model::{AccountIdInternal, NewsCountResult, NewsIteratorSessionId, NewsPage, ResetNewsIteratorResult};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::{Json, StatusCode};
use crate::app::{GetAccounts, ReadData, StateBase, WriteData};


#[obfuscate_api]
const PATH_POST_GET_NEWS_COUNT: &str = "/account_api/news_count";

#[utoipa::path(
    post,
    path = PATH_POST_GET_NEWS_COUNT,
    responses(
        (status = 200, description = "Success.", body = NewsCountResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_news_count<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<NewsCountResult>, StatusCode> {
    ACCOUNT.post_get_news_count.incr();

    let r = state.read().account().news().news_count(id).await?;
    Ok(r.into())
}

#[obfuscate_api]
const PATH_POST_RESET_NEWS_PAGING: &str = "/account_api/news/reset";

#[utoipa::path(
    post,
    path = PATH_POST_RESET_NEWS_PAGING,
    responses(
        (status = 200, description = "Successfull.", body = ResetNewsIteratorResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_news_paging<S: WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ResetNewsIteratorResult>, StatusCode> {
    ACCOUNT.post_reset_news_paging.incr();
    let iterator_session_id = db_write!(state, move |cmds| {
        cmds.account().handle_reset_news_iterator(account_id)
    })?;
    let r = ResetNewsIteratorResult {
        s: iterator_session_id.into(),
    };

    Ok(r.into())
}

#[obfuscate_api]
const PATH_POST_GET_NEXT_NEWS_PAGE: &str = "/account_api/news";

#[utoipa::path(
    post,
    path = PATH_POST_GET_NEXT_NEWS_PAGE,
    request_body(content = NewsIteratorSessionId),
    responses(
        (status = 200, description = "Success.", body = NewsPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_news_page<S: WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<NewsIteratorSessionId>,
) -> Result<Json<NewsPage>, StatusCode> {
    ACCOUNT.post_get_next_news_page.incr();

    let data = state
        .concurrent_write_profile_blocking(
            account_id.as_id(),
            move |cmds| {
                cmds.next_news_iterator_state(account_id, iterator_session_id)
            }
        )
        .await??;

    if let Some(data) = data {
        // Received likes iterator session ID was valid
        let news = state
            .read()
            .account()
            .news()
            .news_page(data)
            .await?;
        Ok(NewsPage {
            news,
            error_invalid_iterator_session_id: false,
        }.into())
    } else {
        Ok(NewsPage {
            news: vec![],
            error_invalid_iterator_session_id: true,
        }.into())
    }
}

pub fn news_router<S: StateBase + GetAccounts + WriteData + ReadData>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_get_news_count::<S>,
        post_reset_news_paging::<S>,
        post_get_next_news_page::<S>,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_NEWS_COUNTERS_LIST,
    post_get_news_count,
    post_reset_news_paging,
    post_get_next_news_page,
);
