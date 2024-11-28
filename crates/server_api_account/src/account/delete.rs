use axum::extract::State;
use model_account::DeleteStatus;
use obfuscate_api_macro::obfuscate_api;
use server_api::S;
use server_api::create_open_api_router;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::utils::{Json, StatusCode};

// TODO(prod): Save all AccountId UUIDs to prevent reusing removed
// AccountId UUID? Save AccountId UUIDs to history database?

#[obfuscate_api]
const PATH_POST_DELETE: &str = "/account_api/delete";

/// Delete account.
///
/// Changes account state to `pending deletion` from all possible states.
/// Previous state will be saved, so it will be possible to stop automatic
/// deletion process.
#[utoipa::path(
    put,
    path = PATH_POST_DELETE,
    responses(
        (status = 200, description = "State changed to 'pending deletion' successfully."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_delete(
    State(_state): State<S>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_delete.incr();
    // TODO
    Ok(())
}

#[obfuscate_api]
const PATH_GET_DELETION_STATUS: &str = "/account_api/delete";

/// Get deletion status.
///
/// Get information when account will be really deleted.
#[utoipa::path(
    get,
    path = PATH_GET_DELETION_STATUS,
    responses(
        (status = 200, description = "Get was successfull.", body = DeleteStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_deletion_status(
    State(_state): State<S>,
) -> Result<Json<DeleteStatus>, StatusCode> {
    ACCOUNT.get_deletion_status.incr();
    // TODO
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

#[obfuscate_api]
const PATH_CANCEL_DELETION: &str = "/account_api/delete";

/// Cancel account deletion.
///
/// Account state will move to previous state.
#[utoipa::path(
    delete,
    path = PATH_CANCEL_DELETION,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_cancel_deletion(
    State(_state): State<S>,
) -> Result<Json<DeleteStatus>, StatusCode> {
    ACCOUNT.delete_cancel_deletion.incr();
    // TODO
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn delete_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_delete,
        get_deletion_status,
        delete_cancel_deletion,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_DELETE_COUNTERS_LIST,
    post_delete,
    get_deletion_status,
    delete_cancel_deletion,
);
