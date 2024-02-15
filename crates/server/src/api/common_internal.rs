//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::{Path, State};
use model::{AccessToken, Account, AccountId};
use simple_backend::create_counters;

use crate::{
    api::utils::{Json, StatusCode},
    app::{GetAccessTokens, GetAccounts, ReadData},
};

// TODO(microservice): Add route for handling login from account server
// TODO(microservice): Add route for receiving state updates from account server


// pub const PATH_INTERNAL_POST_UPDATE_PROFILE_VISIBLITY: &str =
//     "/internal/profile_api/visibility/:account_id/:value";

// #[utoipa::path(
//     post,
//     path = "/internal/profile_api/visiblity/{account_id}/{value}",
//     params(AccountId, BooleanSetting),
//     responses(
//         (status = 200, description = "Visibility update successfull"),
//         (status = 404, description = "No account found."),
//         (status = 500, description = "Internal server error."),
//     ),
// )]
// pub async fn internal_post_update_profile_visibility<
//     S: ReadData + GetAccounts + GetInternalApi + GetAccessTokens + GetConfig + WriteData,
// >(
//     State(state): State<S>,
//     Path(account_id): Path<AccountId>,
//     Path(value): Path<BooleanSetting>,
// ) -> Result<(), StatusCode> {
//     PROFILE_INTERNAL
//         .internal_post_update_profile_visibility
//         .incr();

//     let account_id = state.accounts().get_internal_id(account_id).await?;

//     // TODO: remove this route

//     Ok(())
// }

// create_counters!(
//     AccountInternalCounters,
//     ACCOUNT_INTERNAL,
//     ACCOUNT_INTERNAL_COUNTERS_LIST,
// );
