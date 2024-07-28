//! Public key management related routes

use axum::{extract::{Path, State}, Extension, Router};
use model::{AccountId, AccountIdInternal, GetPublicKey, PublicKeyId, SetPublicKey};
use server_api::{app::{GetAccounts, WriteData}, db_write};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::{ReadData, StateBase};

pub const PATH_GET_PUBLIC_KEY: &str = "/chat_api/public_key/:account_id";

/// Get current public key of some account
#[utoipa::path(
    get,
    path = "/chat_api/public_key/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Success.", body = GetPublicKey),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_public_key<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(requested_id): Path<AccountId>,
) -> Result<Json<GetPublicKey>, StatusCode> {
    CHAT.get_public_key.incr();

    let requested_internal_id = state.get_internal_id(requested_id).await?;
    let key = state.read().chat().get_public_key(requested_internal_id).await?;
    Ok(key.into())
}

pub const PATH_POST_PUBLIC_KEY: &str = "/chat_api/public_key";

/// Replace current public key with a new public key.
/// Returns public key ID number which server increments.
/// This must be called only when needed as this route will
/// fail every time if current public key ID number is i64::MAX.
#[utoipa::path(
    post,
    path = "/chat_api/public_key",
    request_body(content = SetPublicKey),
    responses(
        (status = 200, description = "Success.", body = PublicKeyId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_public_key<S: WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(new_key): Json<SetPublicKey>,
) -> Result<Json<PublicKeyId>, StatusCode> {
    CHAT.post_public_key.incr();

    let new_key = db_write!(state, move |cmds| {
        cmds.chat().set_public_key(id, new_key)
    })?;

    Ok(new_key.into())
}

pub fn public_key_router<S: StateBase + ReadData + WriteData + GetAccounts>(s: S) -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route(PATH_GET_PUBLIC_KEY, get(get_public_key::<S>))
        .route(PATH_POST_PUBLIC_KEY, post(post_public_key::<S>))
        .with_state(s)
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_PUBLIC_KEY_COUNTERS_LIST,
    get_public_key,
    post_public_key,
);
