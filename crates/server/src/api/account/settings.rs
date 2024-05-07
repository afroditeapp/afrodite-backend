use axum::{extract::State, Extension, Router};
use model::{AccountData, AccountIdInternal, AccountState, BooleanSetting, Capabilities, EventToClientInternal, ProfileVisibility};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write, db_write_multiple, utils::{Json, StatusCode}
    },
    app::{GetAccessTokens, GetConfig, GetInternalApi, ReadData, WriteData}, internal_api,
};

pub const PATH_GET_ACCOUNT_DATA: &str = "/account_api/account_data";

/// Get changeable user information to account.
#[utoipa::path(
    get,
    path = "/account_api/account_data",
    responses(
        (status = 200, description = "Request successfull.", body = AccountData),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_data<S: GetAccessTokens + ReadData + WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<AccountData>, StatusCode> {
    ACCOUNT.get_account_data.incr();
    let data = state
        .read()
        .account()
        .account_data(api_caller_account_id)
        .await?;
    Ok(data.into())
}

pub const PATH_POST_ACCOUNT_DATA: &str = "/account_api/account_data";

/// Set changeable user information to account.
#[utoipa::path(
    post,
    path = "/account_api/account_data",
    request_body(content = AccountData),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_data<S: GetAccessTokens + ReadData + WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<AccountData>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_data.incr();
    // TODO: API limits to prevent DoS attacks

    // TODO: Manual email setting should be removed probably and just
    // use the email from sign in with Google or Apple.

    db_write!(state, move |cmds| cmds
        .account()
        .account_data(api_caller_account_id, data))
}

pub const PATH_SETTING_PROFILE_VISIBILITY: &str = "/account_api/settings/profile_visibility";

/// Update current or pending profile visiblity value.
///
/// NOTE: Client uses this in initial setup.
#[utoipa::path(
    put,
    path = "/account_api/settings/profile_visibility",
    request_body(content = BooleanSetting),
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_setting_profile_visiblity<
    S: GetInternalApi + GetConfig + WriteData,
>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(new_value): Json<BooleanSetting>,
) -> Result<(), StatusCode> {
    ACCOUNT.put_setting_profile_visiblity.incr();

    let new_account = db_write_multiple!(state, move |cmds| {
        let new_account = cmds
            .account()
            .update_syncable_account_data(id, None, move |_, _, visiblity| {
                *visiblity = if visiblity.is_pending() {
                    if new_value.value {
                        ProfileVisibility::PendingPublic
                    } else {
                        ProfileVisibility::PendingPrivate
                    }
                } else {
                    if new_value.value {
                        ProfileVisibility::Public
                    } else {
                        ProfileVisibility::Private
                    }
                };
                Ok(())
            }).await?;
        cmds
            .events()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::ProfileVisibilityChanged(
                    new_account.profile_visibility()
                ),
            )
            .await?;
        Ok(new_account)
    })?;

    internal_api::common::sync_account_state(&state, id, new_account.clone()).await?;

    Ok(())
}

pub fn settings_router(s: crate::app::S) -> Router {
    use axum::routing::{get, post, put};

    use crate::app::S;

    Router::new()
        .route(PATH_GET_ACCOUNT_DATA, get(get_account_data::<S>))
        .route(PATH_POST_ACCOUNT_DATA, post(post_account_data::<S>))
        .route(
            PATH_SETTING_PROFILE_VISIBILITY,
            put(put_setting_profile_visiblity::<S>),
        )
        .with_state(s)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_SETTINGS_COUNTERS_LIST,
    get_account_data,
    post_account_data,
    put_setting_profile_visiblity,
);
