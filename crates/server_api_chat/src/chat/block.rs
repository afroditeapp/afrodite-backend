use axum::{extract::State, Extension};
use model_chat::{AccountId, AccountIdInternal, ReceivedBlocksPage, SentBlocksPage};
use obfuscate_api_macro::obfuscate_api;
use server_api::create_open_api_router;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::{Json, StatusCode};
use crate::{
    app::{GetAccounts, ReadData, StateBase, WriteData},
    db_write_multiple,
};

#[obfuscate_api]
const PATH_POST_BLOCK_PROFILE: &str = "/chat_api/block_profile";

/// Block profile
#[utoipa::path(
    post,
    path = PATH_POST_BLOCK_PROFILE,
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_block_profile<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_block_profile.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    db_write_multiple!(state, move |cmds| {
        let changes = cmds.chat().block_profile(id, requested_profile).await?;
        cmds.events()
            .handle_chat_state_changes(changes.sender)
            .await?;
        cmds.events()
            .handle_chat_state_changes(changes.receiver)
            .await?;
        Ok(())
    })?;

    Ok(())
}

#[obfuscate_api]
const PATH_POST_UNBLOCK_PROFILE: &str = "/chat_api/unblock_profile";

/// Unblock profile
#[utoipa::path(
    post,
    path = PATH_POST_UNBLOCK_PROFILE,
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_unblock_profile<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_unblock_profile.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    db_write_multiple!(state, move |cmds| {
        let changes = cmds
            .chat()
            .delete_block(id, requested_profile)
            .await?;
        cmds.events()
            .handle_chat_state_changes(changes.sender)
            .await?;
        cmds.events()
            .handle_chat_state_changes(changes.receiver)
            .await?;
        Ok(())
    })?;

    Ok(())
}

#[obfuscate_api]
const PATH_GET_SENT_BLOCKS: &str = "/chat_api/sent_blocks";

/// Get list of sent blocks
#[utoipa::path(
    get,
    path = PATH_GET_SENT_BLOCKS,
    responses(
        (status = 200, description = "Success.", body = SentBlocksPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sent_blocks<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentBlocksPage>, StatusCode> {
    CHAT.get_sent_blocks.incr();

    let page = state.read().chat().all_sent_blocks(id).await?;
    Ok(page.into())
}

// TODO: Add some block query info, so that server can send sync received blocks
//       list command to client.

#[obfuscate_api]
const PATH_GET_RECEIVED_BLOCKS: &str = "/chat_api/received_blocks";

/// Get list of received blocks
#[utoipa::path(
    get,
    path = PATH_GET_RECEIVED_BLOCKS,
    responses(
        (status = 200, description = "Success.", body = ReceivedBlocksPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_received_blocks<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ReceivedBlocksPage>, StatusCode> {
    CHAT.get_received_blocks.incr();

    let page = state.read().chat().all_received_blocks(id).await?;
    Ok(page.into())
}

pub fn block_router<S: StateBase + GetAccounts + WriteData + ReadData>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_block_profile::<S>,
        post_unblock_profile::<S>,
        get_sent_blocks::<S>,
        get_received_blocks::<S>,
    )
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_BLOCK_COUNTERS_LIST,
    post_block_profile,
    post_unblock_profile,
    get_sent_blocks,
    get_received_blocks,
);
