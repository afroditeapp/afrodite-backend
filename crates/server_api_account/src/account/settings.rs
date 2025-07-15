use axum::{Extension, extract::State};
use model_account::{
    AccountData, AccountIdInternal, BooleanSetting, EventToClientInternal, ProfileVisibility,
};
use server_api::{S, create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    internal_api,
    utils::{Json, StatusCode},
};

const PATH_GET_ACCOUNT_DATA: &str = "/account_api/account_data";

/// Get changeable user information to account.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_DATA,
    responses(
        (status = 200, description = "Request successfull.", body = AccountData),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_data(
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

const PATH_POST_ACCOUNT_DATA: &str = "/account_api/account_data";

/// Set changeable user information to account.
#[utoipa::path(
    post,
    path = PATH_POST_ACCOUNT_DATA,
    request_body(content = AccountData),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_data(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<AccountData>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_data.incr();
    // TODO(prod): API usage limits for some APIs

    // TODO(prod): Manual email setting should be removed probably and just
    // use the email from sign in with Google or Apple.
    // Update: Perhaps create specific route for setting email and
    // allow that only if account state is in initial setup and
    // sign in with login is not used.

    db_write!(state, move |cmds| cmds
        .account()
        .account_data(api_caller_account_id, data)
        .await)
}

const PATH_SETTING_PROFILE_VISIBILITY: &str = "/account_api/settings/profile_visibility";

/// Update current or pending profile visiblity value.
///
/// NOTE: Client uses this in initial setup.
#[utoipa::path(
    put,
    path = PATH_SETTING_PROFILE_VISIBILITY,
    request_body(content = BooleanSetting),
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_setting_profile_visiblity(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(new_value): Json<BooleanSetting>,
) -> Result<(), StatusCode> {
    ACCOUNT.put_setting_profile_visiblity.incr();

    let new_account = db_write!(state, move |cmds| {
        let new_account = cmds
            .account()
            .update_syncable_account_data(id, None, move |_, _, visiblity| {
                *visiblity = if visiblity.is_pending() {
                    if new_value.value {
                        ProfileVisibility::PendingPublic
                    } else {
                        ProfileVisibility::PendingPrivate
                    }
                } else if new_value.value {
                    ProfileVisibility::Public
                } else {
                    ProfileVisibility::Private
                };
                Ok(())
            })
            .await?;
        cmds.events()
            .send_connected_event(id.uuid, EventToClientInternal::AccountStateChanged)
            .await?;
        Ok(new_account)
    })?;

    internal_api::common::sync_account_state(&state, id, new_account.clone()).await?;

    Ok(())
}

const PATH_SETTING_UNLIMITED_LIKES: &str = "/account_api/settings/unlimited_likes";

#[utoipa::path(
    put,
    path = PATH_SETTING_UNLIMITED_LIKES,
    request_body(content = BooleanSetting),
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_setting_unlimited_likes(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(new_value): Json<BooleanSetting>,
) -> Result<(), StatusCode> {
    ACCOUNT.put_setting_unlimited_likes.incr();

    state
        .data_all_access()
        .update_unlimited_likes(id, new_value.value)
        .await?;

    internal_api::common::sync_unlimited_likes(&state, id).await?;

    Ok(())
}

create_open_api_router!(
        fn router_settings,
        get_account_data,
        post_account_data,
        put_setting_profile_visiblity,
        put_setting_unlimited_likes,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_SETTINGS_COUNTERS_LIST,
    get_account_data,
    post_account_data,
    put_setting_profile_visiblity,
    put_setting_unlimited_likes,
);
