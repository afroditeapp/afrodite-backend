use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, ModerationRequest, ModerationRequestContent};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{ReadData, WriteData},
};

pub const PATH_MODERATION_REQUEST: &str = "/media_api/moderation/request";

/// Get current moderation request.
///
#[utoipa::path(
    get,
    path = "/media_api/moderation/request",
    responses(
        (status = 200, description = "Get moderation request was successfull.", body = ModerationRequest),
        (status = 304, description = "No moderation request found."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_moderation_request<S: ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ModerationRequest>, StatusCode> {
    MEDIA.get_moderation_request.incr();

    let request = state
        .read()
        .moderation_request(account_id)
        .await?
        .ok_or(StatusCode::NOT_MODIFIED)?;

    Ok(request.into())
}

/// Create new or override old moderation request.
///
/// Make sure that moderation request has content IDs which points to your own
/// image slots.
///
#[utoipa::path(
    put,
    path = "/media_api/moderation/request",
    request_body(content = ModerationRequestContent),
    responses(
        (status = 200, description = "Sending or updating new image moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or request content was invalid."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_moderation_request<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(moderation_request): Json<ModerationRequestContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_moderation_request.incr();

    db_write!(state, move |cmds| {
        cmds.media()
            .set_moderation_request(account_id, moderation_request)
    })
}

/// Delete current moderation request which is not yet in moderation.
#[utoipa::path(
    delete,
    path = "/media_api/moderation/request",
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_moderation_request<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.delete_moderation_request.incr();

    db_write!(state, move |cmds| {
        cmds.media()
            .delete_moderation_request_not_yet_in_moderation(account_id)
    })
}

pub fn moderation_request_router(s: crate::app::S) -> Router {
    use axum::routing::{delete, get, put};

    use crate::app::S;

    Router::new()
        .route(PATH_MODERATION_REQUEST, get(get_moderation_request::<S>))
        .route(PATH_MODERATION_REQUEST, put(put_moderation_request::<S>))
        .route(
            PATH_MODERATION_REQUEST,
            delete(delete_moderation_request::<S>),
        )
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_MODERATION_REQUEST_COUNTERS_LIST,
    get_moderation_request,
    put_moderation_request,
    delete_moderation_request,
);
