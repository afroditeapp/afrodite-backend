
use axum::{extract::{Path, State}, Extension};
use model::{AccountIdInternal, BooleanSetting, NewsId, NewsLocale, Permissions, UpdateNewsTranslation, UpdateNewsTranslationResult};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, db_write, db_write_multiple, result::WrappedContextExt, DataError};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::{Json, StatusCode};
use crate::app::{ReadData, StateBase, WriteData};

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

#[obfuscate_api]
const PATH_POST_UPDATE_NEWS_TRANSLATION: &str = "/account_api/admin/update_news_translation/{nid}/{locale}";

#[utoipa::path(
    post,
    path = PATH_POST_UPDATE_NEWS_TRANSLATION,
    params(NewsId, NewsLocale),
    request_body(content = UpdateNewsTranslation),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_update_news_translation<S: ReadData + WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(nid): Path<NewsId>,
    Path(locale): Path<NewsLocale>,
    Json(news_translation): Json<UpdateNewsTranslation>,
) -> Result<Json<UpdateNewsTranslationResult>, StatusCode> {
    ACCOUNT.post_update_news_translation.incr();

    if !permissions.some_admin_news_permissions_granted() ||
        !locale.is_supported_locale() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let result = db_write_multiple!(state, move |cmds| {
        let item = cmds
            .read()
            .account_admin()
            .news()
            .news_translations(nid)
            .await?;

        if !permissions.admin_news_edit_all && item.aid_creator != Some(account_id.uuid) {
            return Err(DataError::NotAllowed.report());
        }

        let current_version = item.translations.into_iter().find(|t| t.locale == locale.locale).and_then(|t| t.version);
        if current_version.is_some() && current_version != Some(news_translation.current_version) {
            return Ok(UpdateNewsTranslationResult::error_already_changed());
        }

        cmds.account_admin().news().upsert_news_translation(
            account_id,
            nid,
            locale,
            news_translation,
        ).await?;

        Ok(UpdateNewsTranslationResult::success())
    })?;

    Ok(result.into())
}

#[obfuscate_api]
const PATH_DELETE_NEWS_TRANSLATION: &str = "/account_api/news/{nid}/{locale}";

#[utoipa::path(
    delete,
    path = PATH_DELETE_NEWS_TRANSLATION,
    params(NewsId, NewsLocale),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_news_translation<S: ReadData + WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(nid): Path<NewsId>,
    Path(locale): Path<NewsLocale>,
) -> Result<(), StatusCode> {
    ACCOUNT.delete_news_translation.incr();

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
        cmds.account_admin().news().delete_news_translation(nid, locale)
    )?;

    Ok(())
}

#[obfuscate_api]
const PATH_POST_SET_NEWS_PUBLICITY: &str = "/account_api/set_news_publicity/{nid}";

#[utoipa::path(
    delete,
    path = PATH_POST_SET_NEWS_PUBLICITY,
    params(NewsId),
    request_body(content = BooleanSetting),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_news_publicity<S: ReadData + WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(nid): Path<NewsId>,
    Json(publicity): Json<BooleanSetting>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_set_news_publicity.incr();

    if !permissions.some_admin_news_permissions_granted() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write_multiple!(state, move |cmds| {
        let item = cmds
            .read()
            .account_admin()
            .news()
            .news_translations(nid)
            .await?;

        if !permissions.admin_news_edit_all && item.aid_creator != Some(account_id.uuid) {
            return Err(DataError::NotAllowed.report());
        }

        if item.public == publicity.value {
            return Ok(());
        }

        cmds.account_admin().news().set_news_publicity(
            nid,
            publicity.value,
        ).await?;

        Ok(())
    })?;

    Ok(())
}

pub fn admin_news_router<S: StateBase + WriteData + ReadData>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_create_news_item::<S>,
        delete_news_item::<S>,
        post_update_news_translation::<S>,
        delete_news_translation::<S>,
        post_set_news_publicity::<S>,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_ADMIN_NEWS_COUNTERS_LIST,
    post_create_news_item,
    delete_news_item,
    post_update_news_translation,
    delete_news_translation,
    post_set_news_publicity,
);
