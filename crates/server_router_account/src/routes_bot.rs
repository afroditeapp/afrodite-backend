use axum::{routing::post, Router};
use server_state::S;

use crate::api;

/// Bot API route handlers
pub struct BotApp;

impl BotApp {
    pub fn create_account_server_router(state: S) -> Router {
        Router::new()
            .route(
                api::account_bot::PATH_REGISTER,
                post(api::account_bot::post_register),
            )
            .route(
                api::account_bot::PATH_LOGIN,
                post(api::account_bot::post_login),
            )
            .with_state(state)
    }
}
