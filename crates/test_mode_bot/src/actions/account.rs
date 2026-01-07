use std::{fmt::Debug, time::Duration};

use api_client::{
    apis::{
        account_api::{self, get_account_state, post_account_setup, post_complete_setup},
        account_bot_api::{post_bot_login, post_bot_register, post_remote_bot_login},
    },
    models::{
        Account, AccountStateContainer, BooleanSetting, EventToClient, ProfileVisibility,
        RemoteBotLogin, SetInitialEmail, auth_pair,
    },
};
use async_trait::async_trait;
use base64::Engine;
use error_stack::{Result, ResultExt};
use futures::SinkExt;
use headers::HeaderValue;
use simple_backend_model::VersionNumber;
use test_mode_utils::{client::TestError, server::TEST_ADMIN_ACCESS_EMAIL};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};
use url::Url;
use utils::api::PATH_CONNECT;

use super::{BotAction, BotState, PreviousValue};
use crate::{
    connection::{
        ApiConnection, EventSender, EventSenderAndQuitWatcher, WsConnection, WsStream,
        create_event_channel,
    },
    utils::assert::bot_assert_eq,
};

pub const DEFAULT_AGE: u8 = 30;

#[derive(Debug)]
pub struct Register;

#[async_trait]
impl BotAction for Register {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if state.id.is_some() {
            return Ok(());
        }

        let id = post_bot_register(state.api())
            .await
            .change_context(TestError::ApiRequest)?;
        state.id = Some(id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Login;

#[async_trait]
impl BotAction for Login {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if state.api.is_access_token_available() {
            return Ok(());
        }
        let login_result = if let Some(password) = state.remote_bot_password() {
            post_remote_bot_login(
                state.api(),
                RemoteBotLogin::new(state.account_id()?, password),
            )
            .await
            .change_context(TestError::ApiRequest)?
        } else {
            post_bot_login(state.api(), state.account_id()?)
                .await
                .change_context(TestError::ApiRequest)?
        };

        let auth_pair = if let Some(Some(auth_pair)) = login_result.tokens {
            *auth_pair.clone()
        } else {
            return Err(TestError::ApiRequest.report());
        };

        state.api.set_access_token(auth_pair.access.token.clone());

        let (event_sender, event_receiver, quit_handle) =
            create_event_channel(state.connections.event_info_handle());
        state.connections.set_events(event_receiver);

        let url = state
            .api_urls
            .api_url
            .join(PATH_CONNECT)
            .change_context(TestError::WebSocket)?;
        let connection: Option<WsConnection> =
            connect_websocket(auth_pair, url, state, event_sender.clone())
                .await?
                .into();

        state.connections.set_connections(ApiConnection {
            connection,
            quit_handle,
        });

        Ok(())
    }
}

async fn connect_websocket(
    auth: auth_pair::AuthPair,
    mut url: Url,
    state: &mut BotState,
    events: EventSenderAndQuitWatcher,
) -> Result<WsConnection, TestError> {
    if url.scheme() == "https" {
        url.set_scheme("wss")
            .map_err(|_| TestError::WebSocket.report())?;
    }
    if url.scheme() == "http" {
        url.set_scheme("ws")
            .map_err(|_| TestError::WebSocket.report())?;
    }

    let mut r = url
        .into_client_request()
        .change_context(TestError::WebSocket)?;
    let web_socket_protocol_version: u8 = 1;
    let client_type_number = 3; // Bot client type
    let version = VersionNumber {
        major: 0,
        minor: 0,
        patch: 0,
    };
    let protocol_header_value = format!(
        "v{},t{},c{}_{}_{}_{}",
        web_socket_protocol_version,
        auth.access.token,
        client_type_number,
        version.major,
        version.minor,
        version.patch,
    );
    r.headers_mut().insert(
        http::header::SEC_WEBSOCKET_PROTOCOL,
        HeaderValue::from_str(&protocol_header_value).change_context(TestError::WebSocket)?,
    );
    let (mut stream, _) = tokio_tungstenite::connect_async(r)
        .await
        .change_context(TestError::WebSocket)?;

    let response = stream
        .next()
        .await
        .ok_or(TestError::WebSocket.report())?
        .change_context(TestError::WebSocket)?;
    let update_tokens = match response {
        Message::Binary(refresh_token) => match refresh_token.to_vec().as_slice() {
            [0] => false,
            [1] => true,
            _ => return Err(TestError::WebSocketWrongValue.report()),
        },
        _ => return Err(TestError::WebSocketWrongValue.report()),
    };

    if update_tokens {
        let binary_token = base64::engine::general_purpose::STANDARD
            .decode(auth.refresh.token)
            .change_context(TestError::WebSocket)?;
        stream
            .send(Message::Binary(binary_token.into()))
            .await
            .change_context(TestError::WebSocket)?;

        let refresh_token = stream
            .next()
            .await
            .ok_or(TestError::WebSocket.report())?
            .change_context(TestError::WebSocket)?;
        match refresh_token {
            Message::Binary(refresh_token) => state.refresh_token = Some(refresh_token.into()),
            _ => return Err(TestError::WebSocketWrongValue.report()),
        }

        let access_token = stream
            .next()
            .await
            .ok_or(TestError::WebSocket.report())?
            .change_context(TestError::WebSocket)?;
        match access_token {
            Message::Binary(access_token_bytes) => {
                let access_token =
                    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(access_token_bytes);
                state.api.set_access_token(access_token)
            }
            _ => return Err(TestError::WebSocketWrongValue.report()),
        }
    }

    // Send empty sync data list
    stream
        .send(Message::Binary(vec![].into()))
        .await
        .change_context(TestError::WebSocket)?;

    let task = tokio::spawn(async move {
        let mut events = events;
        let mut ping_timer = tokio::time::interval(Duration::from_secs(60));
        ping_timer.tick().await; // skip the initial tick
        loop {
            tokio::select! {
                _ = events.quit_watcher.recv() => break,
                _ = handle_connection(&mut stream, &events.event_sender) => (),
                _ = ping_timer.tick() => {
                    match stream
                        .send(Message::Ping(vec![].into()))
                        .await {
                            Ok(_) => (),
                            Err(e) => panic!("Sending ping message to websocket failed, error: {e}"),
                        }
                }
            }
        }
    });

    Ok(WsConnection::new(task))
}

async fn handle_connection(stream: &mut WsStream, sender: &EventSender) {
    loop {
        match stream.next().await {
            Some(event) => match event {
                Ok(Message::Text(event)) => {
                    let event: EventToClient =
                        serde_json::from_str(&event).expect("Failed to parse WebSocket event");
                    sender.send_if_sending_enabled(event).await;
                }
                // Connection test message, which does not need a response
                Ok(Message::Binary(data)) if data.is_empty() => (),
                Ok(Message::Pong(_)) => (),
                Ok(_) => {
                    panic!("Unexpected WebSocket message type");
                }
                Err(e) => {
                    panic!("Unexpected WebSocket error, {e}");
                }
            },
            None => {
                panic!("Unexpected WebSocket connection closing");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountState {
    InitialSetup,
    Normal,
    Banned,
    PendingDeletion,
}

impl AccountState {
    fn to_container(self) -> AccountStateContainer {
        match self {
            Self::InitialSetup => AccountStateContainer {
                initial_setup_completed: Some(true),
                banned: None,
                pending_deletion: None,
            },
            Self::Normal => AccountStateContainer {
                initial_setup_completed: None,
                banned: None,
                pending_deletion: None,
            },
            Self::Banned => AccountStateContainer {
                initial_setup_completed: None,
                banned: Some(true),
                pending_deletion: None,
            },
            Self::PendingDeletion => AccountStateContainer {
                initial_setup_completed: None,
                banned: None,
                pending_deletion: Some(true),
            },
        }
    }
}

impl From<AccountStateContainer> for AccountState {
    fn from(value: AccountStateContainer) -> Self {
        if value.pending_deletion.unwrap_or_default() {
            Self::PendingDeletion
        } else if value.banned.unwrap_or_default() {
            Self::Banned
        } else if !value.initial_setup_completed.unwrap_or(true) {
            Self::InitialSetup
        } else {
            Self::Normal
        }
    }
}

impl From<Account> for AccountState {
    fn from(value: Account) -> Self {
        let state: AccountStateContainer = *value.state;
        state.into()
    }
}

#[derive(Debug)]
pub struct AssertAccountState {
    pub account: AccountState,
    pub visibility: Option<ProfileVisibility>,
}

impl AssertAccountState {
    pub const fn account(wanted: AccountState) -> Self {
        Self {
            account: wanted,
            visibility: None,
        }
    }

    pub const fn account_and_visibility(
        wanted: AccountState,
        wanted_visibility: ProfileVisibility,
    ) -> Self {
        Self {
            account: wanted,
            visibility: Some(wanted_visibility),
        }
    }
}

#[async_trait]
impl BotAction for AssertAccountState {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let state = get_account_state(state.api())
            .await
            .change_context(TestError::ApiRequest)?;

        if let Some(wanted_visibility) = self.visibility {
            bot_assert_eq(state.visibility, wanted_visibility)?;
        }

        bot_assert_eq(state.state, self.account.to_container().into())
    }
}

#[derive(Debug)]
pub struct SetAccountSetup {
    admin: bool,
}

impl SetAccountSetup {
    pub const fn new() -> Self {
        Self { admin: false }
    }

    pub const fn admin() -> Self {
        Self { admin: true }
    }
}

impl Default for SetAccountSetup {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BotAction for SetAccountSetup {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let setup = api_client::models::SetAccountSetup {
            birthdate: None,
            is_adult: true,
        };
        post_account_setup(state.api(), setup)
            .await
            .change_context(TestError::ApiRequest)?;

        let email = if self.admin {
            TEST_ADMIN_ACCESS_EMAIL.to_string()
        } else {
            format!("bot{}@example.com", state.task_id)
        };

        account_api::post_initial_email(state.api(), SetInitialEmail { email })
            .await
            .change_context(TestError::ApiRequest)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct CompleteAccountSetup;

#[async_trait]
impl BotAction for CompleteAccountSetup {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        post_complete_setup(state.api())
            .await
            .change_context(TestError::ApiRequest)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct SetProfileVisibility(pub bool);

#[async_trait]
impl BotAction for SetProfileVisibility {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        account_api::put_setting_profile_visiblity(state.api(), BooleanSetting::new(self.0))
            .await
            .change_context(TestError::ApiRequest)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct GetAccount;

#[async_trait]
impl BotAction for GetAccount {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let account = get_account_state(state.api())
            .await
            .change_context(TestError::ApiRequest)?;
        state.previous_value = PreviousValue::Account(account);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}
