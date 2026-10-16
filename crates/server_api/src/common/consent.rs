use axum::{Extension, extract::State};
use model::{AccountIdInternal, DataConsentSettings, UpdateDataConsentSettings};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, WriteData},
    create_open_api_router, db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_DATA_CONSENT_SETTINGS: &str = "/common_api/data_consent_settings";

/// Get current data consent settings.
#[utoipa::path(
    get,
    path = PATH_GET_DATA_CONSENT_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = DataConsentSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_data_consent_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<DataConsentSettings>, StatusCode> {
    COMMON.get_data_consent_settings.incr();

    let settings = state
        .read()
        .common()
        .consent()
        .data_consent_settings(id)
        .await?;

    Ok(settings.into_public().into())
}

const PATH_POST_DATA_CONSENT_SETTINGS: &str = "/common_api/data_consent_settings";

/// Update specific data consent setting fields.
#[utoipa::path(
    post,
    path = PATH_POST_DATA_CONSENT_SETTINGS,
    request_body = UpdateDataConsentSettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_data_consent_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<UpdateDataConsentSettings>,
) -> Result<(), StatusCode> {
    COMMON.post_data_consent_settings.incr();

    db_write!(state, move |cmds| {
        cmds.common()
            .consent()
            .upsert_data_consent_settings(id, settings)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_consent,
    get_data_consent_settings,
    post_data_consent_settings,
);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_CONSENT_COUNTERS_LIST,
    get_data_consent_settings,
    post_data_consent_settings,
);
