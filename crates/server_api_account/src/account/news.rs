use axum::{
    extract::{Path, Query, State},
    Extension,
};
use model_account::{
    AccountIdInternal, GetNewsItemResult, NewsId, NewsIteratorSessionId, NewsLocale, NewsPage,
    PageItemCountForNewPublicNews, PendingNotificationFlags, Permissions, RequireNewsLocale,
    ResetNewsIteratorResult, UnreadNewsCountResult,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::EventManagerProvider, create_open_api_router, db_write, S};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::{Json, StatusCode};
use crate::app::{ReadData, WriteData};

#[obfuscate_api]
const PATH_GET_UNREAD_NEWS_COUNT: &str = "/account_api/news_count";

/// The unread news count for public news.
#[utoipa::path(
    post,
    path = PATH_GET_UNREAD_NEWS_COUNT,
    responses(
        (status = 200, description = "Success.", body = UnreadNewsCountResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_unread_news_count(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<UnreadNewsCountResult>, StatusCode> {
    ACCOUNT.get_unread_news_count.incr();

    let r = state.read().account().news().unread_news_count(id).await?;

    state
        .event_manager()
        .remove_specific_pending_notification_flags_from_cache(
            id,
            PendingNotificationFlags::NEWS_CHANGED,
        )
        .await;

    Ok(r.into())
}

#[obfuscate_api]
const PATH_POST_RESET_NEWS_PAGING: &str = "/account_api/reset_news_paging";

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
pub async fn post_reset_news_paging(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ResetNewsIteratorResult>, StatusCode> {
    ACCOUNT.post_reset_news_paging.incr();
    let r = db_write!(state, move |cmds| {
        cmds.account().news().handle_reset_news_iterator(account_id)
    })?;

    Ok(r.into())
}

/// For admins the first items on the first page are all
/// private news.
#[obfuscate_api]
const PATH_POST_GET_NEXT_NEWS_PAGE: &str = "/account_api/next_news_page";

#[utoipa::path(
    post,
    path = PATH_POST_GET_NEXT_NEWS_PAGE,
    params(NewsLocale),
    request_body(content = NewsIteratorSessionId),
    responses(
        (status = 200, description = "Success.", body = NewsPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_news_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Query(locale): Query<NewsLocale>,
    Json(iterator_session_id): Json<NewsIteratorSessionId>,
) -> Result<Json<NewsPage>, StatusCode> {
    ACCOUNT.post_get_next_news_page.incr();

    let data = state
        .concurrent_write_profile_blocking(account_id.as_id(), move |cmds| {
            cmds.next_news_iterator_state(account_id, iterator_session_id)
        })
        .await??;

    if let Some(data) = data {
        // Session ID is valid
        let (news, n) = state
            .read()
            .account()
            .news()
            .news_page(
                data,
                locale,
                permissions.some_admin_news_permissions_granted(),
            )
            .await?;
        Ok(NewsPage {
            n,
            news,
            error_invalid_iterator_session_id: false,
        }
        .into())
    } else {
        Ok(NewsPage {
            n: PageItemCountForNewPublicNews::default(),
            news: vec![],
            error_invalid_iterator_session_id: true,
        }
        .into())
    }
}

#[obfuscate_api]
const PATH_GET_NEWS_ITEM: &str = "/account_api/news_item/{nid}";

/// Get news item content using specific locale and fallback to locale "en"
/// if news translation is not found.
///
/// If specific locale is not found when [RequireNewsLocale::require_locale]
/// is `true` then [GetNewsItemResult::item] is `None`.
#[utoipa::path(
    get,
    path = PATH_GET_NEWS_ITEM,
    params(NewsId, NewsLocale, RequireNewsLocale),
    responses(
        (status = 200, description = "Success.", body = GetNewsItemResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_news_item(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(nid): Path<NewsId>,
    Query(locale): Query<NewsLocale>,
    Query(require_locale): Query<RequireNewsLocale>,
) -> Result<Json<GetNewsItemResult>, StatusCode> {
    ACCOUNT.get_news_item.incr();

    let mut item = state
        .read()
        .account()
        .news()
        .news_item(nid, locale, require_locale)
        .await?;
    if !permissions.some_admin_news_permissions_granted() {
        if let Some(item) = item.as_mut() {
            item.clear_admin_info();
        }
    }

    let is_public = state.read().account().news().is_public(nid).await?;

    let news = GetNewsItemResult {
        item,
        private: !is_public,
    };
    Ok(news.into())
}

pub fn news_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_get_unread_news_count,
        post_reset_news_paging,
        post_get_next_news_page,
        get_news_item,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_NEWS_COUNTERS_LIST,
    get_unread_news_count,
    post_reset_news_paging,
    post_get_next_news_page,
    get_news_item,
);
