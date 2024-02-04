use axum::{
    body::Body, extract::{Path, Query, State}, Extension, Router
};
use axum_extra::TypedHeader;
use headers::ContentType;
use model::{
    AccountContent, AccountId, AccountIdInternal, ContentAccessCheck, ContentId,
    ContentProcessingId, ContentProcessingState, ContentSlot, NewContentParams, SlotId,
};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{ContentProcessingProvider, GetAccounts, ReadData, WriteData},
    data::{
        write_concurrent::{ConcurrentWriteAction, ConcurrentWriteContentHandle},
        DataError,
    },
};

pub const PATH_GET_CONTENT: &str = "/media_api/content/:account_id/:content_id";

/// Get content data
#[utoipa::path(
    get,
    path = "/media_api/content/{account_id}/{content_id}",
    params(AccountId, ContentId, ContentAccessCheck),
    responses(
        (status = 200, description = "Get content file.", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content<S: ReadData>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
    Query(_access_check): Query<ContentAccessCheck>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    MEDIA.get_content.incr();

    // TODO: Add access restrictions.

    // TODO: Change to use stream when error handling is improved in future axum
    // version. Or check will the connection be closed if there is an error. And
    // set content lenght? Or use ServeFile service from tower middleware.

    let data = state
        .read()
        .media()
        .content_data(account_id, content_id)
        .await?;

    Ok((TypedHeader(ContentType::octet_stream()), data))
}

pub const PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT: &str =
    "/media_api/all_account_media_content/:account_id";

/// Get list of all media content on the server for one account.
#[utoipa::path(
    get,
    path = "/media_api/all_account_media_content/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = AccountContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_account_media_content<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<AccountContent>, StatusCode> {
    MEDIA.get_all_account_media_content.incr();

    // TODO: access restrictions

    let internal_id = state.accounts().get_internal_id(account_id).await?;

    let internal_current_media = state.read().all_account_media_content(internal_id).await?;

    let data = internal_current_media
        .into_iter()
        .map(|m| m.into())
        .collect();

    Ok(AccountContent { data }.into())
}

pub const PATH_PUT_CONTENT_TO_CONTENT_SLOT: &str = "/media_api/content_slot/:slot_id";

/// Set content to content processing slot.
/// Processing ID will be returned and processing of the content
/// will begin.
/// Events about the content processing will be sent to the client.
///
/// The state of the processing can be also queired. The querying is
/// required to receive the content ID.
///
/// Slots from 0 to 6 are available.
///
/// One account can only have one content in upload or processing state.
/// New upload might potentially delete the previous if processing of it is
/// not complete.
///
#[utoipa::path(
    put,
    path = "/media_api/content_slot/{slot_id}",
    params(SlotId, NewContentParams),
    request_body(content = Vec<u8>, content_type = "image/jpeg"),
    responses(
        (status = 200, description = "Image upload was successful.", body = ContentProcessingId),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_content_to_content_slot<S: WriteData + ContentProcessingProvider>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
    Query(new_content_params): Query<NewContentParams>,
    content_data: Body,
) -> Result<Json<ContentProcessingId>, StatusCode> {
    MEDIA.put_content_to_content_slot.incr();

    let slot = TryInto::<ContentSlot>::try_into(slot_number.slot_id as i64)
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let stream = content_data.into_data_stream();

    let content_info = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<error_stack::Result<_, DataError>> = cmds
                .accquire_image(move |cmds: ConcurrentWriteContentHandle| {
                    Box::new(async move { cmds.save_to_tmp(account_id, stream).await })
                })
                .await;
            out
        })
        .await??;

    state
        .content_processing()
        .queue_new_content(account_id, slot, content_info.clone(), new_content_params)
        .await;

    Ok(content_info.processing_id.into())
}

pub const PATH_GET_CONTENT_SLOT_STATE: &str = "/media_api/content_slot/:slot_id";

/// Get state of content slot.
///
/// Slots from 0 to 6 are available.
///
#[utoipa::path(
    get,
    path = "/media_api/content_slot/{slot_id}",
    params(SlotId),
    responses(
        (status = 200, description = "Successful.", body = ContentProcessingState),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content_slot_state<S: ContentProcessingProvider>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
) -> Result<Json<ContentProcessingState>, StatusCode> {
    MEDIA.get_content_slot_state.incr();

    let slot = TryInto::<ContentSlot>::try_into(slot_number.slot_id as i64)
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    if let Some(state) = state.content_processing().get_state(account_id, slot).await {
        Ok(state.into())
    } else {
        Ok(ContentProcessingState::empty().into())
    }
}

pub const PATH_DELETE_CONTENT: &str = "/media_api/content/:account_id/:content_id";

/// Delete content data. Content can be removed after specific time has passed
/// since removing all usage from it (content is not a security image or profile
/// content).
#[utoipa::path(
    delete,
    path = "/media_api/content/{account_id}/{content_id}",
    params(AccountId, ContentId),
    responses(
        (status = 200, description = "Content data deleted."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_content<S: WriteData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.delete_content.incr();

    // TODO: Add access restrictions.

    // TODO: Add database support for keeping track of content usage.

    let internal_id = state.accounts().get_internal_id(account_id).await?;

    db_write!(state, move |cmds| cmds
        .media()
        .delete_content(internal_id, content_id))
}

pub fn content_router(s: crate::app::S) -> Router {
    use axum::routing::{delete, get, put};

    use crate::app::S;

    Router::new()
        .route(PATH_GET_CONTENT, get(get_content::<S>))
        .route(
            PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT,
            get(get_all_account_media_content::<S>),
        )
        .route(
            PATH_PUT_CONTENT_TO_CONTENT_SLOT,
            put(put_content_to_content_slot::<S>),
        )
        .route(
            PATH_GET_CONTENT_SLOT_STATE,
            get(get_content_slot_state::<S>),
        )
        .route(PATH_DELETE_CONTENT, delete(delete_content::<S>))
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_CONTENT_COUNTERS_LIST,
    get_content,
    get_all_account_media_content,
    get_content_slot_state,
    put_content_to_content_slot,
    delete_content,
);
