pub mod profile;

use std::string;

use axum::Json;
use utoipa::{OpenApi};


use self::profile::{RegisterBody, RegisterResponse, LoginBody, LoginResponse};

use tracing::{error, info};

use super::GetDatabaseTaskSender;

#[derive(OpenApi)]
#[openapi(
    paths(
        register,
        login,
    ),
    components(
        schemas(
            crate::api::ApiResult,
            crate::api::ApiResultEnum,
            profile::RegisterBody,
            profile::RegisterResponse,
            profile::LoginBody,
            profile::LoginResponse,
        )
    )
)]
pub struct ApiDocCore;


// TODO: Add timeout for database commands


pub const PATH_REGISTER: &str = "/register";

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterBody,
    responses(
        (
            status = 200,
            description = "Register new profile",
            body = [RegisterResponse],
        ),
    )
)]
pub async fn register<S: GetDatabaseTaskSender>(
    Json(profile_info): Json<RegisterBody>,
    mut state: S,
) -> Json<RegisterResponse> {
    match state.database().send_command(profile_info).await.await.unwrap() {
        Ok(response) => response.into(),
        Err(e) => {
            error!("Database task error: {:?}", e);
            RegisterResponse::database_error().into()
        }
    }
}

pub const PATH_LOGIN: &str = "/login";

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginBody,
    responses(
        (
            status = 200,
            description = "Get API key for this profile",
            body = [LoginResponse],
        ),
    )
)]
pub async fn login<S: GetDatabaseTaskSender>(
    Json(profile_info): Json<LoginBody>,
    mut state: S,
) -> Json<LoginResponse> {
    match state.database().send_command(profile_info).await.await.unwrap() {
        Ok(response) => response.into(),
        Err(e) => {
            error!("Database task error: {:?}", e);
            LoginResponse::database_error().into()
        }
    }
}
