//! Public key management related routes

use axum::{
    body::{to_bytes, Body}, extract::{Path, Query, State}, Extension
};
use model::Permissions;
use model_chat::{
    AccountId, AccountIdInternal, AddPublicKeyResult, GetPrivatePublicKeyInfo, PublicKeyId
};
use server_api::{
    app::{GetAccounts, WriteData}, create_open_api_router, db_write_multiple, S
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_PUBLIC_KEY: &str = "/chat_api/public_key/{aid}";

/// Get current public key of some account
#[utoipa::path(
    get,
    path = PATH_GET_PUBLIC_KEY,
    params(AccountId, PublicKeyId),
    responses(
        (status = 200, description = "Success.", body = inline(model::BinaryData), content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_public_key(
    State(state): State<S>,
    Path(requested_id): Path<AccountId>,
    Query(key_id): Query<PublicKeyId>,
) -> Result<Vec<u8>, StatusCode> {
    CHAT.get_public_key.incr();

    let requested_internal_id = state.get_internal_id(requested_id).await?;
    let key = state
        .read()
        .chat()
        .public_key()
        .get_public_key_data(requested_internal_id, key_id)
        .await?;

    if let Some(key_data) = key {
        Ok(key_data)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

const PATH_POST_ADD_PUBLIC_KEY: &str = "/chat_api/add_public_key";

/// Add new public key.
///
/// Returns next public key ID number.
///
/// # Limits
///
/// Server can store limited amount of public keys. The limit is
/// configurable from server config file and also user specific config exists.
/// Max value between the two previous values is used to check is adding the
/// key allowed.
///
#[utoipa::path(
    post,
    path = PATH_POST_ADD_PUBLIC_KEY,
    request_body(content = inline(model::BinaryData), content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Success.", body = AddPublicKeyResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_add_public_key(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    key_data: Body,
) -> Result<Json<AddPublicKeyResult>, StatusCode> {
    CHAT.post_add_public_key.incr();

    let key_data = to_bytes(key_data, 1024 * 8)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_vec();

    let new_key = db_write_multiple!(state, move |cmds| {
        cmds.chat().add_public_key(id, key_data).await
    })?;

    Ok(new_key.into())
}

const PATH_GET_PRIVATE_PUBLIC_KEY_INFO: &str = "/chat_api/private_public_key_info/{aid}";

/// Get private public key info
///
/// # Access
/// * Owner of the requested account
/// * Permission [model::Permissions::admin_edit_max_public_key_count]
#[utoipa::path(
    get,
    path = PATH_GET_PRIVATE_PUBLIC_KEY_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Success.", body = GetPrivatePublicKeyInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_private_public_key_info(
    State(state): State<S>,
    Extension(api_caller): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Path(requested_id): Path<AccountId>,
) -> Result<Json<GetPrivatePublicKeyInfo>, StatusCode> {
    CHAT.get_private_public_key_info.incr();

    let access_allowed =
        api_caller.as_id() == requested_id ||
        api_caller_permissions.admin_edit_max_public_key_count;

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_internal_id = state.get_internal_id(requested_id).await?;
    let info = state
        .read()
        .chat()
        .public_key()
        .get_private_public_key_info(requested_internal_id)
        .await?;
    Ok(info.into())
}

create_open_api_router!(fn router_public_key, get_public_key, post_add_public_key, get_private_public_key_info,);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_PUBLIC_KEY_COUNTERS_LIST,
    get_public_key,
    post_add_public_key,
    get_private_public_key_info,
);
