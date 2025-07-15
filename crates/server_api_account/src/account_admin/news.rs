use axum::{
    Extension,
    extract::{Path, State},
};
use model_account::{
    AccountIdInternal, BooleanSetting, NewsId, NewsLocale, NotificationEvent, Permissions,
    UpdateNewsTranslation, UpdateNewsTranslationResult,
};
use server_api::{DataError, S, create_open_api_router, db_write, result::WrappedContextExt};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::{ReadData, WriteData};

const PATH_POST_CREATE_NEWS_ITEM: &str = "/account_api/create_news_item";

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
pub async fn post_create_news_item(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<NewsId>, StatusCode> {
    ACCOUNT.post_create_news_item.incr();

    if !permissions.admin_news_create {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let news_id = db_write!(state, move |cmds| cmds
        .account_admin()
        .news()
        .create_news_item(account_id)
        .await)?;
    Ok(news_id.into())
}

const PATH_DELETE_NEWS_ITEM: &str = "/account_api/delete_news/{nid}";

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
pub async fn delete_news_item(
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

    db_write!(state, move |cmds| cmds
        .account_admin()
        .news()
        .delete_news_item(nid)
        .await)?;

    Ok(())
}

const PATH_POST_UPDATE_NEWS_TRANSLATION: &str =
    "/account_api/update_news_translation/{nid}/{locale}";

#[utoipa::path(
    post,
    path = PATH_POST_UPDATE_NEWS_TRANSLATION,
    params(NewsId, NewsLocale),
    request_body(content = UpdateNewsTranslation),
    responses(
        (status = 200, description = "Success.", body = UpdateNewsTranslationResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_update_news_translation(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(nid): Path<NewsId>,
    Path(locale): Path<NewsLocale>,
    Json(news_translation): Json<UpdateNewsTranslation>,
) -> Result<Json<UpdateNewsTranslationResult>, StatusCode> {
    ACCOUNT.post_update_news_translation.incr();

    if !permissions.some_admin_news_permissions_granted() || !locale.is_supported_locale() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let result = db_write!(state, move |cmds| {
        let item = cmds
            .read()
            .account_admin()
            .news()
            .news_translations(nid)
            .await?;

        if !permissions.admin_news_edit_all && item.aid_creator != Some(account_id.uuid) {
            return Err(DataError::NotAllowed.report());
        }

        let current_version = item
            .translations
            .into_iter()
            .find(|t| t.locale == locale.locale)
            .and_then(|t| t.version);
        if current_version.is_some() && current_version != Some(news_translation.current_version) {
            return Ok(UpdateNewsTranslationResult::error_already_changed());
        }

        cmds.account_admin()
            .news()
            .upsert_news_translation(account_id, nid, locale, news_translation)
            .await?;

        Ok(UpdateNewsTranslationResult::success())
    })?;

    Ok(result.into())
}

const PATH_DELETE_NEWS_TRANSLATION: &str = "/account_api/delete_news_translation/{nid}/{locale}";

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
pub async fn delete_news_translation(
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

    db_write!(state, move |cmds| cmds
        .account_admin()
        .news()
        .delete_news_translation(nid, locale)
        .await)?;

    Ok(())
}

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
pub async fn post_set_news_publicity(
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

    db_write!(state, move |cmds| {
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

        cmds.account_admin()
            .news()
            .set_news_publicity(nid, publicity.value)
            .await?;

        cmds.events()
            .send_low_priority_notification_to_logged_in_clients(NotificationEvent::NewsChanged)
            .await;

        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_news,
        post_create_news_item,
        delete_news_item,
        post_update_news_translation,
        delete_news_translation,
        post_set_news_publicity,
);

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
