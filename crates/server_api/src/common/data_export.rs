use std::str::FromStr;

use axum::{
    Extension,
    body::Body,
    extract::{Path, Query, State},
};
use axum_extra::TypedHeader;
use headers::{ContentLength, ContentType};
use model::{
    AccountId, AccountIdInternal, DataExportName, DataExportState, DataExportStateType, Permissions,
};
use server_data::{
    data_export::{SourceAccount, TargetAccount},
    read::GetReadCommandsCommon,
    write::GetWriteCommandsCommon,
};
use server_state::db_write;
use simple_backend::create_counters;

use crate::{
    S,
    app::{DataExportManagerDataProvider, GetAccounts, ReadData, WriteData},
    create_open_api_router,
    utils::{Json, StatusCode},
};

const PATH_GET_DATA_EXPORT_STATE: &str = "/common_api/data_export_state";

#[utoipa::path(
    get,
    path = PATH_GET_DATA_EXPORT_STATE,
    responses(
        (status = 200, description = "Successfull.", body = DataExportState),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_data_export_state(
    State(state): State<S>,
    Extension(api_caller_id): Extension<AccountIdInternal>,
) -> Result<Json<DataExportState>, StatusCode> {
    COMMON.get_data_export_state.incr();
    let value = state
        .data_export()
        .get_state(api_caller_id)
        .await
        .get_public_state()
        .await;
    Ok(value.into())
}

const PATH_POST_START_DATA_EXPORT: &str = "/common_api/start_data_export/{aid}";

/// Start data export
///
/// Data export state will move from [DataExportStateType::Empty] to
/// [DataExportStateType::InProgress].
///
/// # Access
///
/// * Without admin permission, own account can exported once per 24 hours.
///   The export command sending time is stored only in RAM, so the limit
///   resets when backend restarts.
/// * With [Permissions::admin_export_data] all accounts can be exported
///   without limits.
///
#[utoipa::path(
    post,
    path = PATH_POST_START_DATA_EXPORT,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_start_data_export(
    State(state): State<S>,
    Extension(api_caller_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Path(account_for_exporting): Path<AccountId>,
) -> Result<(), StatusCode> {
    COMMON.post_start_data_export.incr();

    let export_state = state.data_export().get_state(api_caller_id).await;

    let account_for_exporting = SourceAccount(state.get_internal_id(account_for_exporting).await?);
    let api_caller_id = TargetAccount(api_caller_id);

    if api_caller_permissions.admin_export_data {
        state
            .data_export()
            .send_export_cmd_if_export_file_is_not_in_use(account_for_exporting, api_caller_id)
            .await?;
        return Ok(());
    }

    if !export_state
        .enough_time_elapsed_since_previous_export()
        .await
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    if account_for_exporting.0 != api_caller_id.0 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    state
        .data_export()
        .send_export_cmd_if_export_file_is_not_in_use(account_for_exporting, api_caller_id)
        .await?;

    Ok(())
}

const PATH_DELETE_DATA_EXPORT: &str = "/common_api/delete_data_export";

/// Delete current data export
///
/// Data export state will move from [DataExportStateType::Done] or
/// [DataExportStateType::Error] to [DataExportStateType::Empty].
#[utoipa::path(
    delete,
    path = PATH_DELETE_DATA_EXPORT,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_data_export(
    State(state): State<S>,
    Extension(api_caller_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    COMMON.delete_data_export.incr();

    state
        .data_export()
        .delete_state_if_export_not_ongoing(api_caller_id)
        .await?;

    // Account tmp dir is cleared every server restart so it does not matter if
    // db_write does not run.

    db_write!(state, move |cmds| {
        cmds.common()
            .data_export()
            .delete_data_export(api_caller_id)
            .await
    })?;

    Ok(())
}

const PATH_GET_DATA_EXPORT_ARCHIVE: &str = "/common_api/data_export_archive";

/// Download current data export archive
///
/// Requires data export state [DataExportStateType::Done].
#[utoipa::path(
    get,
    path = PATH_GET_DATA_EXPORT_ARCHIVE,
    params(DataExportName),
    responses(
        (status = 200, description = "Successfull.", body = inline(model::BinaryData), content_type = "application/zip"),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_data_export_archive(
    State(state): State<S>,
    Extension(api_caller_id): Extension<AccountIdInternal>,
    Query(export_name): Query<DataExportName>,
) -> Result<(TypedHeader<ContentType>, TypedHeader<ContentLength>, Body), StatusCode> {
    COMMON.get_data_export_archive.incr();

    let export_state = state
        .data_export()
        .get_state(api_caller_id)
        .await
        .get_public_state()
        .await;

    if export_state.name != Some(export_name) || export_state.state != DataExportStateType::Done {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let (byte_count, data_stream) = state
        .read()
        .common()
        .data_export()
        .data_export_archive_stream(api_caller_id)
        .await?;

    let content_type: ContentType = FromStr::from_str("application/zip").unwrap();

    Ok((
        TypedHeader(content_type),
        TypedHeader(ContentLength(byte_count)),
        Body::from_stream(data_stream),
    ))
}

create_open_api_router!(
    fn router_data_export,
    get_data_export_state,
    post_start_data_export,
    delete_data_export,
    get_data_export_archive,
);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_DATA_EXPORT_COUNTERS_LIST,
    get_data_export_state,
    post_start_data_export,
    delete_data_export,
    get_data_export_archive,
);
