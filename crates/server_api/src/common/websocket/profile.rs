use axum::extract::ws::WebSocket;
use model::{AccountIdInternal, EventToClientInternal, ResponseNextProfilePageStatus};
use model_server_data::ProfileIteratorSessionId;
use server_common::websocket::WebSocketError;
use server_state::S;

use super::send_event;
use crate::common::get_next_profile_page;

pub async fn handle_get_next_profile_page(
    state: &S,
    socket: &mut WebSocket,
    account_id: AccountIdInternal,
    iterator_session_id: ProfileIteratorSessionId,
) -> crate::result::Result<(), WebSocketError> {
    match get_next_profile_page(account_id, iterator_session_id, state).await {
        Ok(page) => {
            let status = if page.error_invalid_iterator_session_id_value() {
                ResponseNextProfilePageStatus::InvalidIteratorSessionId
            } else {
                ResponseNextProfilePageStatus::Success
            };

            let profiles = if matches!(status, ResponseNextProfilePageStatus::Success) {
                page.profiles().to_vec()
            } else {
                Vec::new()
            };

            send_event(
                socket,
                EventToClientInternal::ResponseNextProfilePage { status, profiles },
            )
            .await?;
        }
        Err(crate::utils::StatusCode::TOO_MANY_REQUESTS) => {
            send_event(
                socket,
                EventToClientInternal::ResponseNextProfilePage {
                    status: ResponseNextProfilePageStatus::RateLimited,
                    profiles: Vec::new(),
                },
            )
            .await?;
        }
        Err(_) => {
            send_event(
                socket,
                EventToClientInternal::ResponseNextProfilePage {
                    status: ResponseNextProfilePageStatus::InternalServerError,
                    profiles: Vec::new(),
                },
            )
            .await?;
        }
    }

    Ok(())
}
