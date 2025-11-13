use std::time::Duration;

use model::AccountIdInternal;
use server_common::websocket::WebSocketError;
use server_data::{read::GetReadCommandsCommon, result::WrappedResultExt};
use server_state::{S, state_impl::ReadData};
use simple_backend::perf::websocket::{self, ConnectionTracker};
use tokio::time::Instant;

pub struct ConnectionPingTracker {
    timer: tokio::time::Interval,
}

impl ConnectionPingTracker {
    const TIMEOUT_IN_SECONDS: u64 = 60 * 6;

    pub fn new() -> Self {
        let first_tick = Instant::now() + Duration::from_secs(Self::TIMEOUT_IN_SECONDS);
        Self {
            timer: tokio::time::interval_at(
                first_tick,
                Duration::from_secs(Self::TIMEOUT_IN_SECONDS),
            ),
        }
    }

    pub async fn wait_timeout(&mut self) {
        self.timer.tick().await;
    }

    pub async fn reset(&mut self) {
        self.timer.reset();
    }
}

pub struct WebSocketConnectionTrackers {
    _all: ConnectionTracker,
    _gender_specific: Option<ConnectionTracker>,
}

impl WebSocketConnectionTrackers {
    pub async fn new(
        state: &S,
        id: AccountIdInternal,
    ) -> crate::result::Result<Self, WebSocketError> {
        let info = state
            .read()
            .common()
            .bot_and_gender_info(id)
            .await
            .change_context(WebSocketError::DatabaseBotAndGenderInfoQuery)?;

        let all = if info.is_bot {
            websocket::BotConnections::create().into()
        } else {
            websocket::Connections::create().into()
        };

        let gender_specific = if info.is_bot {
            if info.gender.is_man() {
                Some(websocket::BotConnectionsMen::create().into())
            } else if info.gender.is_woman() {
                Some(websocket::BotConnectionsWomen::create().into())
            } else if info.gender.is_non_binary() {
                Some(websocket::BotConnectionsNonbinaries::create().into())
            } else {
                None
            }
        } else if info.gender.is_man() {
            Some(websocket::ConnectionsMen::create().into())
        } else if info.gender.is_woman() {
            Some(websocket::ConnectionsWomen::create().into())
        } else if info.gender.is_non_binary() {
            Some(websocket::ConnectionsNonbinaries::create().into())
        } else {
            None
        };

        Ok(Self {
            _all: all,
            _gender_specific: gender_specific,
        })
    }
}
