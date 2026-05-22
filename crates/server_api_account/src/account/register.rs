use axum::{Extension, extract::State};
use model_account::{AccountIdInternal, AccountState};
use server_api::{S, create_open_api_router};
use simple_backend::create_counters;

use crate::utils::StatusCode;

const PATH_ACCOUNT_COMPLETE_SETUP: &str = "/account_api/complete_setup";

/// Complete initial setup.
///
/// Media content with InSlot state will be removed.
///
/// Requirements:
///  - Account must be in `InitialSetup` state.
///
#[utoipa::path(
    post,
    path = PATH_ACCOUNT_COMPLETE_SETUP,
    responses(
        (status = 200, description = "Request successfull."),
        (status = 406, description = "Current state is not initial setup."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or current state is invalid for Normal state."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_complete_setup(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_complete_setup.incr();

    // Initial account state check. The complete setup implementation has
    // another which handles race conditions.
    if account_state != AccountState::InitialSetup {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    state.data_all_access().complete_initial_setup(id).await?;

    Ok(())
}

create_open_api_router!(
        /// Contains only routes which require authentication.
        fn router_register,
        post_complete_setup,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REGISTER_COUNTERS_LIST,
    post_complete_setup,
);
