use axum::extract::ws::WebSocket;
use model::{
    AccountIdInternal, EventToClientInternal, ResponseNextProfilePageStatus,
    ResponseResetProfilePagingStatus,
};
use model_server_data::{AutomaticProfileSearchIteratorSessionId, ProfileIteratorSessionId};
use server_common::websocket::WebSocketError;
use server_state::S;

use super::send_event;
use crate::common::{
    automatic_profile_search_get_next_profile_page, automatic_profile_search_reset_profile_paging,
    get_next_profile_page, reset_profile_paging,
};

pub async fn handle_get_next_profile_page(
    state: &S,
    socket: &mut WebSocket,
    account_id: AccountIdInternal,
    request_id: u8,
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
                EventToClientInternal::ResponseNextProfilePage {
                    request_id,
                    status,
                    profiles,
                },
            )
            .await?;
        }
        Err(crate::utils::StatusCode::TOO_MANY_REQUESTS) => {
            send_event(
                socket,
                EventToClientInternal::ResponseNextProfilePage {
                    request_id,
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
                    request_id,
                    status: ResponseNextProfilePageStatus::InternalServerError,
                    profiles: Vec::new(),
                },
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn handle_reset_profile_paging(
    state: &S,
    socket: &mut WebSocket,
    account_id: AccountIdInternal,
    request_id: u8,
) -> crate::result::Result<(), WebSocketError> {
    match reset_profile_paging(account_id, state).await {
        Ok(iterator_session_id) => {
            send_event(
                socket,
                EventToClientInternal::ResponseResetProfilePaging {
                    request_id,
                    status: ResponseResetProfilePagingStatus::Success,
                    iterator_session_id: Some(iterator_session_id.as_i64()),
                },
            )
            .await?;
        }
        Err(crate::utils::StatusCode::TOO_MANY_REQUESTS) => {
            send_event(
                socket,
                EventToClientInternal::ResponseResetProfilePaging {
                    request_id,
                    status: ResponseResetProfilePagingStatus::RateLimited,
                    iterator_session_id: None,
                },
            )
            .await?;
        }
        Err(_) => {
            send_event(
                socket,
                EventToClientInternal::ResponseResetProfilePaging {
                    request_id,
                    status: ResponseResetProfilePagingStatus::InternalServerError,
                    iterator_session_id: None,
                },
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn handle_automatic_profile_search_reset_profile_paging(
    state: &S,
    socket: &mut WebSocket,
    account_id: AccountIdInternal,
    request_id: u8,
) -> crate::result::Result<(), WebSocketError> {
    match automatic_profile_search_reset_profile_paging(account_id, state).await {
        Ok(iterator_session_id) => {
            send_event(
                socket,
                EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
                    request_id,
                    status: ResponseResetProfilePagingStatus::Success,
                    iterator_session_id: Some(iterator_session_id.as_i64()),
                },
            )
            .await?;
        }
        Err(crate::utils::StatusCode::TOO_MANY_REQUESTS) => {
            send_event(
                socket,
                EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
                    request_id,
                    status: ResponseResetProfilePagingStatus::RateLimited,
                    iterator_session_id: None,
                },
            )
            .await?;
        }
        Err(_) => {
            send_event(
                socket,
                EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
                    request_id,
                    status: ResponseResetProfilePagingStatus::InternalServerError,
                    iterator_session_id: None,
                },
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn handle_automatic_profile_search_get_next_profile_page(
    state: &S,
    socket: &mut WebSocket,
    account_id: AccountIdInternal,
    request_id: u8,
    iterator_session_id: AutomaticProfileSearchIteratorSessionId,
) -> crate::result::Result<(), WebSocketError> {
    match automatic_profile_search_get_next_profile_page(account_id, iterator_session_id, state)
        .await
    {
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
                EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
                    request_id,
                    status,
                    profiles,
                },
            )
            .await?;
        }
        Err(crate::utils::StatusCode::TOO_MANY_REQUESTS) => {
            send_event(
                socket,
                EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
                    request_id,
                    status: ResponseNextProfilePageStatus::RateLimited,
                    profiles: Vec::new(),
                },
            )
            .await?;
        }
        Err(_) => {
            send_event(
                socket,
                EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
                    request_id,
                    status: ResponseNextProfilePageStatus::InternalServerError,
                    profiles: Vec::new(),
                },
            )
            .await?;
        }
    }

    Ok(())
}
