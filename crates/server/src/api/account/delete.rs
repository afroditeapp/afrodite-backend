
use axum::{Extension, extract::State, Router};
use model::{
    AccessToken, Account, AccountData, AccountId, AccountIdInternal, AccountSetup, AccountState,
    AuthPair, BooleanSetting, DeleteStatus, EventToClientInternal, GoogleAccountId, LoginResult,
    RefreshToken, SignInWithInfo, SignInWithLoginInfo,
};
use simple_backend::{app::SignInWith, create_counters};
use tracing::error;

use crate::api::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{
    app::{
        EventManagerProvider, GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData,
        WriteData,
    },
};



pub const PATH_POST_DELETE: &str = "/account_api/delete";

/// Delete account.
///
/// Changes account state to `pending deletion` from all possible states.
/// Previous state will be saved, so it will be possible to stop automatic
/// deletion process.
#[utoipa::path(
    put,
    path = "/account_api/delete",
    responses(
        (status = 200, description = "State changed to 'pending deletion' successfully."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_delete<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_delete.incr();
    // TODO
    Ok(())
}

pub const PATH_GET_DELETION_STATUS: &str = "/account_api/delete";

/// Get deletion status.
///
/// Get information when account will be really deleted.
#[utoipa::path(
    get,
    path = "/account_api/delete",
    responses(
        (status = 200, description = "Get was successfull.", body = DeleteStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_deletion_status<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
) -> Result<Json<DeleteStatus>, StatusCode> {
    ACCOUNT.get_deletion_status.incr();
    // TODO
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub const PATH_CANCEL_DELETION: &str = "/account_api/delete";

/// Cancel account deletion.
///
/// Account state will move to previous state.
#[utoipa::path(
    delete,
    path = "/account_api/delete",
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_cancel_deletion<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
) -> Result<Json<DeleteStatus>, StatusCode> {
    ACCOUNT.delete_cancel_deletion.incr();
    // TODO
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn delete_router(s: crate::app::S) -> Router {
    use crate::app::S;
    use axum::routing::{get, post, delete};

    Router::new()
        .route(PATH_POST_DELETE, post(post_delete::<S>))
        .route(PATH_GET_DELETION_STATUS, get(get_deletion_status::<S>))
        .route(PATH_CANCEL_DELETION, delete(delete_cancel_deletion::<S>))
        .with_state(s)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_DELETE_COUNTERS_LIST,
    post_delete,
    get_deletion_status,
    delete_cancel_deletion,
);
