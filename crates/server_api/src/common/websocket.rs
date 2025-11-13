use model::{AccountIdInternal, EventToServer, EventToServerType};
use server_common::websocket::WebSocketError;
use server_data::{app::ReadData, db_manager::InternalReading};
use server_state::S;

pub mod chat;
pub mod tracker;

/// Errors which can cause log spam are ignored so
/// logging the returned error is safe.
pub async fn handle_event_to_server(
    state: &S,
    id: AccountIdInternal,
    text: &str,
) -> crate::result::Result<(), WebSocketError> {
    let Ok(msg): Result<EventToServer, _> = serde_json::from_str(text) else {
        // Ignore invalid message
        return Ok(());
    };

    match msg.message_type() {
        EventToServerType::TypingStart => {
            let Some(typing_to) = msg.account() else {
                // Ignore invalid message
                return Ok(());
            };
            let Some(typing_to) = state
                .read()
                .cache()
                .to_account_id_internal_optional(typing_to)
                .await
            else {
                // Ignore invalid account ID
                return Ok(());
            };
            chat::handle_typing_start(state, id, typing_to).await
        }
        EventToServerType::TypingStop => chat::handle_typing_stop(state, id).await,
    }
}
