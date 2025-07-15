use axum::{Extension, extract::State};
use model_chat::{AccountId, AccountIdInternal, SentBlocksPage};
use server_api::{S, create_open_api_router};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write,
};

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
pub async fn post_block_profile(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_block_profile.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().block_profile(id, requested_profile).await?;
        Ok(())
    })?;

    Ok(())
}

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
pub async fn post_unblock_profile(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_unblock_profile.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().delete_block(id, requested_profile).await?;
        Ok(())
    })?;

    Ok(())
}

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
pub async fn get_sent_blocks(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentBlocksPage>, StatusCode> {
    CHAT.get_sent_blocks.incr();

    let page = state.read().chat().all_sent_blocks(id).await?;
    Ok(page.into())
}

create_open_api_router!(
        fn router_block,
        post_block_profile,
        post_unblock_profile,
        get_sent_blocks,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_BLOCK_COUNTERS_LIST,
    post_block_profile,
    post_unblock_profile,
    get_sent_blocks,
);
