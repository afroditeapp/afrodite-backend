use axum::{extract::State, Extension, Router};
use model::{AccountData, AccountIdInternal, AccountState, BooleanSetting, EventToClientInternal};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{EventManagerProvider, GetAccessTokens, GetConfig, GetInternalApi, ReadData, WriteData},
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

/// Update profile visiblity value.
///
/// This will check that the first image moderation request has been moderated
/// before this turns the profile public.
///
/// Sets capablity `view_public_profiles` on or off depending on the value.
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
    S: GetAccessTokens + ReadData + GetInternalApi + GetConfig + WriteData + EventManagerProvider,
>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(new_value): Json<BooleanSetting>,
) -> Result<(), StatusCode> {
    ACCOUNT.put_setting_profile_visiblity.incr();
    let account = state.read().account().account(id).await?;

    if account.state() != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let new_capabilities = state
        .internal_api()
        .modify_and_sync_account_state(id, |d| {
            d.capabilities.user_view_public_profiles = new_value.value;
            *d.is_profile_public = new_value.value;
        })
        .await?;

    state
        .event_manager()
        .send_connected_event(
            id.uuid,
            EventToClientInternal::AccountCapabilitiesChanged {
                capabilities: new_capabilities,
            },
        )
        .await?;

    // TODO could this be removed, because there is already the sync call above?
    state
        .internal_api()
        .profile_api_set_profile_visiblity(id, new_value)
        .await?;

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
