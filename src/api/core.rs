pub mod profile;
pub mod user;

use axum::{Json, middleware::Next, response::Response, extract::Path};
use hyper::{StatusCode, Request};
use tokio::sync::Mutex;
use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, ApiKeyValue}};

use crate::server::session::UserState;

use self::{
    profile::Profile,
    user::{UserId, ApiKey},
};

use tracing::{error, info};

use super::{GetSessionManager, GetRouterDatabaseHandle, GetUsers, GetApiKeys};

#[derive(OpenApi)]
#[openapi(
    paths(register, login, profile),
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
                    utoipa::openapi::security::ApiKey::Header(ApiKeyValue::new(API_KEY_HEADER))
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
pub async fn register<S: GetRouterDatabaseHandle + GetUsers>(
    state: S,
) -> Result<Json<UserId>, StatusCode> {
    // New unique UUID is generated every time so no special handling needed.
    let new_user_id = UserId::new(uuid::Uuid::new_v4().simple().to_string());

    let mut write_commands = state.database().user_write_commands(&new_user_id);
    match write_commands.register().await {
        Ok(()) => {
            state.users()
                .write()
                .await
                .insert(new_user_id.clone(), Mutex::new(write_commands));
            Ok(new_user_id.into())
        }
        Err(e) => {
            error!("Error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
pub async fn login<S: GetApiKeys + GetUsers>(
    Json(user_id): Json<UserId>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {
    // TODO: check that UserId contains only hexadecimals

    let key = ApiKey::new(uuid::Uuid::new_v4().simple().to_string());

    state
        .users()
        .read()
        .await
        .get(&user_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)? // User does not exists.
        .lock()
        .await
        .update_current_api_key(&key)
        .await
        .map_err(|e| {
            error!("Login error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    let user_state = UserState::new(user_id);
    state.api_keys()
        .write()
        .await
        .insert(key.clone(), user_state);


    Ok(key.into())
}

pub const PATH_PROFILE: &str = "/profile/:user_id";

#[utoipa::path(
    get,
    path = "/profile/{user_id}",
    params(UserId),
    responses(
        (status = 200, description = "Get profile.", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn profile<S: GetSessionManager>(
    Path(user_id): Path<UserId>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: Validate user id
    state.session_manager()
        .get_profile(user_id).await
        .map(|profile| profile.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}


const API_KEY_HEADER: &str = "X-API-KEY";

pub async fn authenticate<T, S: GetSessionManager>(
    session_manager: S,
    req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req.headers().get(API_KEY_HEADER).ok_or(StatusCode::UNAUTHORIZED)?;
    let key_str = header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let key = ApiKey::new(key_str.to_string());

    if session_manager.session_manager().api_key_is_valid(key).await {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
