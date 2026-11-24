use std::time::Duration;

use axum::{Extension, extract::State};
use model::{AccountIdInternal, FeedbackConfig};
use model_account::{SendFeedback, SendFeedbackResult};
use server_api::{
    S,
    app::{ApiLimitsProvider, GetConfig},
    create_open_api_router,
};
use simple_backend::create_counters;
use tokio::time::timeout;
use unicode_segmentation::UnicodeSegmentation;

use crate::utils::{Json, StatusCode};

const PATH_GET_FEEDBACK_CONFIG: &str = "/account_api/feedback_config";

/// Get feedback form config.
///
/// Returns an empty category list when the feedback feature is not
/// enabled in the server config.
#[utoipa::path(
    get,
    path = PATH_GET_FEEDBACK_CONFIG,
    responses(
        (status = 200, description = "Successful.", body = FeedbackConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_feedback_config(
    State(state): State<S>,
) -> Result<Json<FeedbackConfig>, StatusCode> {
    ACCOUNT.get_feedback_config.incr();

    let config = state
        .config()
        .feedback()
        .map(|v| v.to_client_config())
        .unwrap_or_default();

    Ok(config.into())
}

const PATH_POST_SEND_FEEDBACK: &str = "/account_api/send_feedback";

/// Send feedback to the server admins.
///
/// The feedback is sent as an email to the admin email address configured
/// in the server config. The user's current email address is added to the
/// `Reply-To` header.
///
/// The category key is validated against the current server config, so a
/// removed category returns `error_unknown_category` and the client should
/// fetch the feedback config again.
#[utoipa::path(
    post,
    path = PATH_POST_SEND_FEEDBACK,
    request_body = SendFeedback,
    responses(
        (status = 200, description = "Successful.", body = SendFeedbackResult),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_feedback(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(request): Json<SendFeedback>,
) -> Result<Json<SendFeedbackResult>, StatusCode> {
    ACCOUNT.post_send_feedback.incr();

    let Some(feedback_config) = state.config().feedback() else {
        return Ok(SendFeedbackResult::unknown_category().into());
    };

    if feedback_config.category(&request.category_key).is_none() {
        return Ok(SendFeedbackResult::unknown_category().into());
    }

    let body = request.body.into_string();

    let body_grapheme_cluster_count = body.graphemes(true).count();
    if body_grapheme_cluster_count > feedback_config.body_max_grapheme_cluster_count.get() as usize
    {
        return Ok(SendFeedbackResult::body_too_long().into());
    }

    state
        .api_limits(account_id)
        .account()
        .send_feedback()
        .await?;

    let handle =
        state
            .email_channel_sender()
            .send_feedback(account_id, request.category_key, body)?;

    let result = timeout(Duration::from_secs(10), handle.wait()).await;

    match result {
        Ok(Ok(())) => Ok(SendFeedbackResult::success().into()),
        Ok(Err(_)) | Err(_) => Ok(SendFeedbackResult::email_sending_failed().into()),
    }
}

create_open_api_router!(
        fn router_feedback,
        get_feedback_config,
        post_send_feedback,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_FEEDBACK_COUNTERS_LIST,
    get_feedback_config,
    post_send_feedback,
);
