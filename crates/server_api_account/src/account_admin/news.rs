
use axum::{extract::{Path, State}, Extension};
use model::{AccountIdInternal, NewsId, Permissions};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::{Json, StatusCode};
use crate::app::{GetAccounts, ReadData, StateBase, WriteData};

#[obfuscate_api]
const PATH_POST_CREATE_NEWS_ITEM: &str = "/account_api/admin/create_news_item";

#[utoipa::path(
    post,
    path = PATH_POST_CREATE_NEWS_ITEM,
    responses(
        (status = 200, description = "Success.", body = NewsId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_create_news_item<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<NewsId>, StatusCode> {
    ACCOUNT.post_create_news_item.incr();

    if !permissions.admin_news_create {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let news_id = db_write!(state, move |cmds|
        cmds.account_admin().news().create_news_item(account_id)
    )?;
    Ok(news_id.into())
}

#[obfuscate_api]
const PATH_DELETE_NEWS_ITEM: &str = "/account_api/news/{nid}";

#[utoipa::path(
    delete,
    path = PATH_DELETE_NEWS_ITEM,
    params(NewsId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_news_item<S: ReadData + WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(nid): Path<NewsId>,
) -> Result<(), StatusCode> {
    ACCOUNT.delete_news_item.incr();

    if !permissions.some_admin_news_permissions_granted() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let item = state
        .read()
        .account_admin()
        .news()
        .news_translations(nid)
        .await?;

    if !permissions.admin_news_edit_all && item.aid_creator != Some(account_id.uuid) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds|
        cmds.account_admin().news().delete_news_item(nid)
    )?;

    Ok(())
}

pub fn admin_news_router<S: StateBase + GetAccounts + WriteData + ReadData>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_create_news_item::<S>,
        delete_news_item::<S>,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_ADMIN_NEWS_COUNTERS_LIST,
    post_create_news_item,
    delete_news_item,
);
