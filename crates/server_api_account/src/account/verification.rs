use axum::{Extension, extract::State};
use model::{AccountIdInternal, AdminBotNotificationTypes};
use model_account::{
    AccountVerificationQueueStatus, PostAccountVerificationQueueItem,
    PostAccountVerificationQueueItemResult,
};
use server_api::{
    AccountVerificationQueueAddError, S,
    app::{
        AccountVerificationQueueProvider, AdminNotificationProvider, ApiLimitsProvider, GetConfig,
        ReadData,
    },
    create_open_api_router,
};
use server_data::read::GetReadCommandsCommon;
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_ACCOUNT_VERIFICATION_QUEUE_STATUS: &str = "/account_api/account_verification_queue";

/// Get account verification queue status for current account.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_VERIFICATION_QUEUE_STATUS,
    responses(
        (status = 200, description = "Successful.", body = AccountVerificationQueueStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_verification_queue_status(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<AccountVerificationQueueStatus>, StatusCode> {
    ACCOUNT.get_account_verification_queue_status.incr();

    let queue_position = state
        .account_verification_queue()
        .queue_position(api_caller_account_id)
        .await;

    let verification_data = state
        .read()
        .account()
        .account_verification_data(api_caller_account_id)
        .await?;

    Ok(AccountVerificationQueueStatus {
        queue_position,
        verification_method: verification_data.verification_method,
        verification_unix_time: verification_data.verification_unix_time,
        verification_error_flags: verification_data.verification_error_flags,
    }
    .into())
}

const PATH_POST_ACCOUNT_VERIFICATION_QUEUE_ITEM: &str = "/account_api/account_verification_queue";

/// Add account verification request to queue for current account.
///
/// Adding new request requires initial setup to be completed.
#[utoipa::path(
    post,
    path = PATH_POST_ACCOUNT_VERIFICATION_QUEUE_ITEM,
    request_body = PostAccountVerificationQueueItem,
    responses(
        (status = 200, description = "Successful.", body = PostAccountVerificationQueueItemResult),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_verification_queue_item(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<PostAccountVerificationQueueItem>,
) -> Result<Json<PostAccountVerificationQueueItemResult>, StatusCode> {
    ACCOUNT.post_account_verification_queue_item.incr();

    state
        .api_limits(api_caller_account_id)
        .account()
        .post_account_verification_queue_item()
        .await?;

    let account = state.read().common().account(api_caller_account_id).await?;
    if !account.state_container().initial_setup_completed() {
        return Ok(
            PostAccountVerificationQueueItemResult::error_initial_setup_not_completed().into(),
        );
    }

    let max_queue_length = state
        .config()
        .limits_account()
        .account_verification_queue_max_length;

    let add_result = state
        .account_verification_queue()
        .add(api_caller_account_id, data, max_queue_length)
        .await;

    let result = match add_result {
        Ok(()) => {
            state
                .admin_notification()
                .send_bot_notification_if_needed(AdminBotNotificationTypes::VERIFY_ACCOUNT_BOT)
                .await;

            PostAccountVerificationQueueItemResult::success()
        }
        Err(AccountVerificationQueueAddError::AlreadyQueued) => {
            PostAccountVerificationQueueItemResult::error_already_in_queue()
        }
        Err(AccountVerificationQueueAddError::QueueFull) => {
            PostAccountVerificationQueueItemResult::error_queue_full()
        }
    };

    Ok(result.into())
}

create_open_api_router!(
    fn router_verification,
    get_account_verification_queue_status,
    post_account_verification_queue_item,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_VERIFICATION_COUNTERS_LIST,
    get_account_verification_queue_status,
    post_account_verification_queue_item,
);
