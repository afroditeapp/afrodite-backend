//! Public key management related routes

use axum::{Extension, extract::State};
use model::Permissions;
use model_chat::SetMaxPublicKeyCount;
use server_api::{
    S,
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple,
};
use server_data_chat::write::GetWriteCommandsChat;
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};

const PATH_POST_SET_MAX_PUBLIC_KEY_COUNT: &str = "/chat_api/set_max_public_key_count";

/// Set max public key count
///
/// # Access
/// * Permission [model::Permissions::admin_edit_max_public_key_count]
#[utoipa::path(
    post,
    path = PATH_POST_SET_MAX_PUBLIC_KEY_COUNT,
    request_body = SetMaxPublicKeyCount,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_set_max_public_key_count(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(info): Json<SetMaxPublicKeyCount>,
) -> Result<(), StatusCode> {
    CHAT.post_set_max_public_key_count.incr();

    if !api_caller_permissions.admin_edit_max_public_key_count {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let id = state.get_internal_id(info.account).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.chat_admin()
            .public_key()
            .set_max_public_key_count(id, info.count)
            .await
    })?;

    Ok(())
}

create_open_api_router!(fn router_admin_public_key, post_set_max_public_key_count,);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_ADMIN_PUBLIC_KEY_COUNTERS_LIST,
    post_set_max_public_key_count,
);
