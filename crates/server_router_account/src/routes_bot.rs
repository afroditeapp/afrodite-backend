use axum::{Router, routing::post};
use server_state::S;

use crate::api;

/// Bot API route handlers
pub struct PublicBotApp;

impl PublicBotApp {
    pub fn create_account_server_router(state: S) -> Router {
        Router::new()
            .route(
                api::account_bot::PATH_REMOTE_BOT_LOGIN,
                post(api::account_bot::post_remote_bot_login),
            )
            .with_state(state)
    }
}

/// Bot API route handlers
pub struct BotApp;

impl BotApp {
    pub fn create_account_server_router(state: S) -> Router {
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
                api::account_bot::PATH_REMOTE_BOT_LOGIN,
                post(api::account_bot::post_remote_bot_login),
            )
            .with_state(state)
    }
}
