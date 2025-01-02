use axum::{
    body::Body,
    extract::{Path, Query, State},
    Extension,
};
use axum_extra::TypedHeader;
use headers::{ContentLength, ContentType};
use model::EventToClientInternal;
use model_media::{
    AccountContent, AccountId, AccountIdInternal, AccountState, ContentId, ContentProcessingId,
    ContentProcessingState, ContentSlot, GetContentQueryParams, NewContentParams, Permissions,
    SlotId,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::GetConfig, create_open_api_router, db_write_multiple, result::WrappedResultExt, S};
use server_data::{
    read::GetReadCommandsCommon,
    write_concurrent::{ConcurrentWriteAction, ConcurrentWriteContentHandle},
    DataError,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{ContentProcessingProvider, GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_CONTENT: &str = "/media_api/content/{aid}/{cid}";

/// Get content data
///
/// # Access
///
/// ## Own content
/// Unrestricted access.
///
/// ## Public other content
/// Normal account state required.
///
/// ## Private other content
/// If owner of the requested content is a match and the requested content
/// is in current profile content, then the requested content can be accessed
/// if query parameter `is_match` is set to `true`.
///
/// If the previous is not true, then permission `admin_view_all_profiles` or
/// `admin_moderate_profile_content` is required.
///
#[utoipa::path(
    get,
    path = PATH_GET_CONTENT,
    params(AccountId, ContentId, GetContentQueryParams),
    responses(
        (status = 200, description = "Get content file.", body = inline(model::BinaryData), content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(permissions): Extension<Permissions>,
    Path(requested_profile): Path<AccountId>,
    Path(requested_content_id): Path<ContentId>,
    Query(params): Query<GetContentQueryParams>,
) -> Result<(TypedHeader<ContentType>, TypedHeader<ContentLength>, Body), StatusCode> {
    MEDIA.get_content.incr();

    let send_content = || async {
        let data = state
            .read()
            .media()
            .content_data(requested_profile, requested_content_id)
            .await?;

        let (lenght, stream) = data
            .byte_count_and_read_stream()
            .await
            .change_context(DataError::File)?;

        Ok((
            TypedHeader(ContentType::octet_stream()),
            TypedHeader(ContentLength(lenght)),
            Body::from_stream(stream),
        ))
    };

    if account_id.as_id() == requested_profile {
        return send_content().await;
    }

    if account_state != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_profile_internal_id = state.get_internal_id(requested_profile).await?;

    let visibility = state
        .read()
        .common()
        .account(requested_profile_internal_id)
        .await?
        .profile_visibility()
        .is_currently_public();

    let internal = state
        .read()
        .media()
        .current_account_media(requested_profile_internal_id)
        .await?;

    let requested_content_is_profile_content = internal
        .iter_current_profile_content()
        .any(|c| c.content_id() == requested_content_id);

    if (visibility && requested_content_is_profile_content)
        || permissions.admin_view_all_profiles
        || permissions.admin_moderate_profile_content
        || (params.is_match
            && requested_content_is_profile_content
            && state
                .data_all_access()
                .is_match(account_id, requested_profile_internal_id)
                .await?)
    {
        send_content().await
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[obfuscate_api]
const PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT: &str = "/media_api/all_account_media_content/{aid}";

/// Get list of all media content on the server for one account.
///
/// # Access
///
/// - Own account
#[utoipa::path(
    get,
    path = PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = AccountContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_account_media_content(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<AccountContent>, StatusCode> {
    MEDIA.get_all_account_media_content.incr();

    let internal_id = state.get_internal_id(account_id).await?;

    let access_allowed = api_caller_account_id == internal_id;
    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_current_media = state
        .read()
        .media()
        .all_account_media_content(internal_id)
        .await?;

    let data = internal_current_media
        .into_iter()
        .map(|m| m.into())
        .collect();

    Ok(AccountContent {
        data,
        max_content_count: state.config().limits_media().max_content_count,
        unused_content_wait_seconds: state.config().limits_media().unused_content_wait_time_seconds.seconds,
    }.into())
}

#[obfuscate_api]
const PATH_PUT_CONTENT_TO_CONTENT_SLOT: &str = "/media_api/content_slot/{slot_id}";

/// Upload content to server. The content is saved to content processing
/// slot when account state is [model::AccountState::InitialSetup].
/// In other states the slot number is ignored and content goes
/// directly to moderation.
///
/// Processing ID will be returned and processing of the content
/// will begin. Events about the content processing will be sent
/// to the client.
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
/// Content processing will fail if image content resolution width or height
/// value is less than 512.
///
#[utoipa::path(
    put,
    path = PATH_PUT_CONTENT_TO_CONTENT_SLOT,
    params(SlotId, NewContentParams),
    request_body(content = inline(model::BinaryData), content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Image upload was successful.", body = ContentProcessingId),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_content_to_content_slot(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
    Query(new_content_params): Query<NewContentParams>,
    content_data: Body,
) -> Result<Json<ContentProcessingId>, StatusCode> {
    MEDIA.put_content_to_content_slot.incr();

    let slot = TryInto::<ContentSlot>::try_into(slot_number.slot_id as i64)
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let count = state.read().media().all_account_media_content_count(account_id).await?;
    if count > state.config().limits_media().max_content_count.into() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let stream = content_data.into_data_stream();

    let content_info = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<crate::result::Result<_, DataError>> = cmds
                .accquire_image(move |cmds: ConcurrentWriteContentHandle| {
                    Box::new(async move { cmds.save_to_tmp(account_id, slot, stream).await })
                })
                .await;
            out
        })
        .await??;

    state
        .content_processing()
        .queue_new_content(account_id, slot, content_info.clone(), new_content_params)
        .await?;

    Ok(content_info.processing_id.into())
}

#[obfuscate_api]
const PATH_GET_CONTENT_SLOT_STATE: &str = "/media_api/content_slot/{slot_id}";

/// Get state of content slot.
///
/// Slots from 0 to 6 are available.
///
#[utoipa::path(
    get,
    path = PATH_GET_CONTENT_SLOT_STATE,
    params(SlotId),
    responses(
        (status = 200, description = "Successful.", body = ContentProcessingState),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content_slot_state(
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

#[obfuscate_api]
const PATH_DELETE_CONTENT: &str = "/media_api/content/{aid}/{cid}";

/// Delete content data.
///
/// # Own account
/// Content can be deleted after specific time has passed
/// since removing all usage of it (content is not assigned
/// as security or profile content).
///
/// # Admin
/// Admin can remove content without restrictions with
/// permission `admin_delete_media_content`.
#[utoipa::path(
    delete,
    path = PATH_DELETE_CONTENT,
    params(AccountId, ContentId),
    responses(
        (status = 200, description = "Content data deleted."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_content(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(content_owner_account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.delete_content.incr();

    let content_owner_account_id = state.get_internal_id(content_owner_account_id).await?;
    let content_id = state.read().media().content_id_internal(content_owner_account_id, content_id).await?;
    let content = state.read().media().content_state(content_id).await?;

    if *content_owner_account_id.as_db_id() != content.account_id {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let owner_deleting_content = content_owner_account_id == api_caller_account_id;
    let admin_access = permissions.admin_delete_media_content;
    let route_access_allowed = owner_deleting_content || admin_access;

    if !route_access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if owner_deleting_content && !admin_access && !content.removable_by_user(state.config().limits_media().unused_content_wait_time_seconds.seconds) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write_multiple!(state, move |cmds| {
        let r = cmds
            .media()
            .delete_content(content_id)
            .await?;

        if r.current_media_content_refresh_needed {
            cmds.events()
                .send_connected_event(
                    api_caller_account_id,
                    EventToClientInternal::MediaContentChanged,
                )
                .await?;
        }

        Ok(())
    })
}

pub fn content_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_content,
        get_all_account_media_content,
        put_content_to_content_slot,
        get_content_slot_state,
        delete_content,
    )
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
