use axum::{Extension, extract::State};
use model_account::{AccountIdInternal, BooleanSetting, EventToClientInternal, ProfileVisibility};
use server_api::{S, create_open_api_router, db_write};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

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

    db_write!(state, move |cmds| {
        let new_account = cmds
            .account()
            .update_syncable_account_data(id, None, move |_, _, visiblity, _| {
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

    Ok(())
}

create_open_api_router!(
        fn router_settings,
        put_setting_profile_visiblity,
        put_setting_unlimited_likes,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_SETTINGS_COUNTERS_LIST,
    put_setting_profile_visiblity,
    put_setting_unlimited_likes,
);
