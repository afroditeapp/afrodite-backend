use axum::{Extension, extract::State};
use model::Permissions;
use model_account::{
    AccountVerificationQueueAdminItem, GetAccountVerificationQueueNextItemResult,
    PostAccountVerificationQueueRemoveNextItem,
};
use server_api::{
    S,
    app::{AccountVerificationQueueProvider, EventManagerProvider, GetAccounts},
    create_open_api_router,
};
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_ACCOUNT_VERIFICATION_QUEUE_NEXT_ITEM: &str =
    "/account_api/account_verification_queue_next_item";

/// Get next item in account verification queue.
///
/// # Access
/// * Permission [model::Permissions::admin_verify_account]
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_VERIFICATION_QUEUE_NEXT_ITEM,
    responses(
        (status = 200, description = "Successful", body = GetAccountVerificationQueueNextItemResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_verification_queue_next_item(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetAccountVerificationQueueNextItemResult>, StatusCode> {
    ACCOUNT_ADMIN
        .get_account_verification_queue_next_item
        .incr();

    if !permissions.admin_verify_account {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let item = state
        .account_verification_queue()
        .next_item()
        .await
        .map(|(account_id, value)| AccountVerificationQueueAdminItem {
            account_id,
            verification_method: value.verification_method,
            verification_data: value.verification_data,
        });

    Ok(GetAccountVerificationQueueNextItemResult { item }.into())
}

const PATH_POST_ACCOUNT_VERIFICATION_QUEUE_REMOVE_NEXT_ITEM: &str =
    "/account_api/account_verification_queue_remove_next_item";

/// Remove next item from account verification queue if possible.
///
/// Removal succeeds only when the provided account id matches queue head item owner.
/// No error is returned if there is a mismatch.
///
/// # Access
/// * Permission [model::Permissions::admin_verify_account]
#[utoipa::path(
    post,
    path = PATH_POST_ACCOUNT_VERIFICATION_QUEUE_REMOVE_NEXT_ITEM,
    request_body = PostAccountVerificationQueueRemoveNextItem,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_verification_queue_remove_next_item(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<PostAccountVerificationQueueRemoveNextItem>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN
        .post_account_verification_queue_remove_next_item
        .incr();

    if !permissions.admin_verify_account {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let expected_account_id = state.get_internal_id(data.account_id).await?;

    let _ = state
        .account_verification_queue()
        .remove_next_item(expected_account_id, &state.event_manager())
        .await;

    Ok(())
}

create_open_api_router!(
    fn router_admin_verification,
    get_account_verification_queue_next_item,
    post_account_verification_queue_remove_next_item,
);

create_counters!(
    AccountAdminCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_VERIFICATION_COUNTERS_LIST,
    get_account_verification_queue_next_item,
    post_account_verification_queue_remove_next_item,
);
