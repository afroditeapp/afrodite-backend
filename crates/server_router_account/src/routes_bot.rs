use axum::{Router, routing::post};
use server_state::S;

use crate::api;

pub struct RemoteBotApiRoutes;

impl RemoteBotApiRoutes {
    pub fn router(state: S) -> Router {
        Router::new()
            .route(
                api::account_bot::PATH_REMOTE_BOT_LOGIN,
                post(api::account_bot::post_remote_bot_login),
            )
            .route(
                api::account_bot::PATH_REMOTE_GET_BOTS,
                post(api::account_bot::post_remote_get_bots),
            )
            .with_state(state)
    }
}

pub struct LocalBotApiRoutes;

impl LocalBotApiRoutes {
    pub fn router(state: S) -> Router {
        Router::new()
            .route(
                api::account_bot::PATH_BOT_REGISTER,
                post(api::account_bot::post_bot_register),
            )
            .route(
                api::account_bot::PATH_BOT_LOGIN,
                post(api::account_bot::post_bot_login),
            )
            .route(
                api::account_bot::PATH_GET_BOTS,
                post(api::account_bot::post_get_bots),
            )
            .with_state(state)
    }
}
