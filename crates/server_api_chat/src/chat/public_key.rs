//! Public key management related routes

use axum::{
    extract::{Path, Query, State},
    Extension,
};
use model_chat::{
    AccountId, AccountIdInternal, GetPublicKey, PublicKeyId, PublicKeyVersion, SetPublicKey,
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
    params(AccountId, PublicKeyVersion),
    responses(
        (status = 200, description = "Success.", body = GetPublicKey),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_public_key(
    State(state): State<S>,
    Path(requested_id): Path<AccountId>,
    Query(key_version): Query<PublicKeyVersion>,
) -> Result<Json<GetPublicKey>, StatusCode> {
    CHAT.get_public_key.incr();

    let requested_internal_id = state.get_internal_id(requested_id).await?;
    let key = state
        .read()
        .chat()
        .get_public_key(requested_internal_id, key_version)
        .await?;
    Ok(key.into())
}

const PATH_POST_PUBLIC_KEY: &str = "/chat_api/public_key";

/// Replace current public key with a new public key.
/// Returns public key ID number which server increments.
/// This must be called only when needed as this route will
/// fail every time if current public key ID number is i64::MAX.
///
/// Only version 1 public keys are currently supported.
#[utoipa::path(
    post,
    path = PATH_POST_PUBLIC_KEY,
    request_body(content = SetPublicKey),
    responses(
        (status = 200, description = "Success.", body = PublicKeyId),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unsupported public key version"),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_public_key(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(new_key): Json<SetPublicKey>,
) -> Result<Json<PublicKeyId>, StatusCode> {
    CHAT.post_public_key.incr();

    if new_key.version.version != 1 {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    let new_key = db_write_multiple!(state, move |cmds| {
        cmds.chat().set_public_key(id, new_key).await
    })?;

    Ok(new_key.into())
}

create_open_api_router!(fn router_public_key, get_public_key, post_public_key,);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_PUBLIC_KEY_COUNTERS_LIST,
    get_public_key,
    post_public_key,
);
