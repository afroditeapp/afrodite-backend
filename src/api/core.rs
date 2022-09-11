pub mod profile;
pub mod user;

use axum::Json;
use hyper::StatusCode;
use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, ApiKeyValue}};

use self::{
    profile::Profile,
    user::{UserId, ApiKey},
};

use tracing::{error, info};

use super::GetSessionManager;

#[derive(OpenApi)]
#[openapi(
    paths(register, login),
    components(schemas(
        user::UserId,
        user::ApiKey,
        profile::Profile,
    )),
    modifiers(&SecurityApiTokenDefault),
)]
pub struct ApiDocCore;

struct SecurityApiTokenDefault;
impl Modify for SecurityApiTokenDefault {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(ApiKeyValue::new("example_key"))
                ),
            )
        }
    }
}


// TODO: Add timeout for database commands

pub const PATH_REGISTER: &str = "/register";

#[utoipa::path(
    post,
    path = "/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = [UserId]),
        (status = 500),
    )
)]
pub async fn register<S: GetSessionManager>(
    state: S,
) -> Result<Json<UserId>, StatusCode> {
    state.session_manager()
        .register().await
        .map(|user_id| user_id.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub const PATH_LOGIN: &str = "/login";

#[utoipa::path(
    post,
    path = "/login",
    security(),
    request_body = UserId,
    responses(
        (status = 200, description = "Login successful.", body = [ApiKey]),
        (status = 500),
    ),
)]
pub async fn login<S: GetSessionManager>(
    Json(user_id): Json<UserId>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {
    state.session_manager()
        .login(user_id).await
        .map(|token| token.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub const PATH_PROFILE: &str = "/profile";

// #[utoipa::path(
//     get,
//     path = "/profile/{id}",
//     responses(
//         (status = 200, description = "Get profile.", body = [Profile]),
//         (status = 500),
//     ),
// )]
// pub async fn profile<S: GetSessionManager>(
//     mut state: S,
// ) -> Result<Json<Profile>, > {
//     state.session_manager()
//         .get_profile(profile_info.into_user_id()).await
//         .map(|token| TokenJson::new(token).into())
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
// }
