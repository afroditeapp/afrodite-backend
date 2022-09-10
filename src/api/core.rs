pub mod profile;
pub mod user;

use std::string;

use axum::Json;
use utoipa::OpenApi;

use self::{
    profile::ProfileResponse,
    user::{LoginBody, LoginResponse, RegisterBody, RegisterResponse},
};

use tracing::{error, info};

use super::GetSessionManager;

#[derive(OpenApi)]
#[openapi(
    paths(register, login, profile,),
    components(schemas(
        crate::api::ApiResult,
        crate::api::ApiResultEnum,
        user::RegisterBody,
        user::RegisterResponse,
        user::LoginBody,
        user::LoginResponse,
    ))
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
pub async fn register<S: GetSessionManager>(
    Json(profile_info): Json<RegisterBody>,
    state: S,
) -> Json<RegisterResponse> {
    match state.session_manager().register().await {
        Ok(user_id) => RegisterResponse::success(user_id),
        Err(()) => RegisterResponse::database_error(),
    }
    .into()
}

pub const PATH_LOGIN: &str = "/login";

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginBody,
    responses(
        (
            status = 200,
            description = "Get API token for this profile",
            body = [LoginResponse],
        ),
    )
)]
pub async fn login<S: GetSessionManager>(
    Json(profile_info): Json<LoginBody>,
    state: S,
) -> Json<LoginResponse> {
    match state.session_manager().login(profile_info.user_id).await {
        Ok(api_token) => LoginResponse::success(api_token),
        Err(()) => LoginResponse::database_error(),
    }
    .into()
}

pub const PATH_PROFILE: &str = "/profile";

#[utoipa::path(
    post,
    path = "/profile",
    responses(
        (
            status = 200,
            description = "Get your profile.",
            body = [ProfileResponse],
        ),
    ),
)]
pub async fn profile<S: GetSessionManager>(
    //Json(profile_info): Json<Pro>,
    mut state: S,
) -> Json<ProfileResponse> {
    ProfileResponse::database_error().into()
}
