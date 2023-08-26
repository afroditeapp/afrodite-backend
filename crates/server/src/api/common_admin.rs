//! Common routes related to admin features

use axum::{extract::Query, Extension};
use manager_model::{
    BuildInfo, RebootQueryParam, SoftwareInfo, SoftwareOptionsQueryParam, SystemInfoList,
};
use model::{Account, AccountIdInternal};

use crate::api::{
    utils::{Json, StatusCode},
    GetManagerApi, ReadData,
};

pub const PATH_GET_SYSTEM_INFO: &str = "/common_api/system_info";

/// Get system information from manager instance.
#[utoipa::path(
    get,
    path = "/common_api/system_info",
    responses(
        (status = 200, description = "Get was successfull.", body = SystemInfoList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_system_info<S: GetManagerApi + ReadData>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SystemInfoList>, StatusCode> {
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

    if account.capablities().admin_server_maintentance_view_info {
        let info = state.manager_api().system_info().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_GET_SOFTWARE_INFO: &str = "/common_api/software_info";

/// Get software version information from manager instance.
#[utoipa::path(
    get,
    path = "/common_api/software_info",
    responses(
        (status = 200, description = "Get was successfull.", body = SoftwareInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_software_info<S: GetManagerApi + ReadData>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SoftwareInfo>, StatusCode> {
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

    if account.capablities().admin_server_maintentance_view_info {
        let info = state.manager_api().software_info().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_GET_LATEST_BUILD_INFO: &str = "/common_api/get_latest_build_info";

/// Get latest software build information available for update from manager
/// instance.
#[utoipa::path(
    get,
    path = "/common_api/get_latest_build_info",
    params(SoftwareOptionsQueryParam),
    responses(
        (status = 200, description = "Get was successfull.", body = BuildInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_build_info<S: GetManagerApi + ReadData>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<BuildInfo>, StatusCode> {
    let account: Account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

    if account.capablities().admin_server_maintentance_view_info {
        let info = state
            .manager_api()
            .get_latest_build_info(software.software_options)
            .await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_REQUEST_BUILD_SOFTWARE: &str = "/common_api/request_build_software";

/// Request building new software from manager instance.
#[utoipa::path(
    post,
    path = "/common_api/request_build_software",
    params(SoftwareOptionsQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_build_software<S: GetManagerApi + ReadData>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<(), StatusCode> {
    let account: Account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

    if account
        .capablities()
        .admin_server_maintentance_update_software
    {
        state
            .manager_api()
            .request_build_software_from_build_server(software.software_options)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_REQUEST_UPDATE_SOFTWARE: &str = "/common_api/request_update_software";

/// Request updating new software from manager instance.
#[utoipa::path(
    post,
    path = "/common_api/request_update_software",
    params(SoftwareOptionsQueryParam, RebootQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_update_software<S: GetManagerApi + ReadData>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Query(reboot): Query<RebootQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<(), StatusCode> {
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

    if account
        .capablities()
        .admin_server_maintentance_update_software
    {
        state
            .manager_api()
            .request_update_software(software.software_options, reboot.reboot)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
