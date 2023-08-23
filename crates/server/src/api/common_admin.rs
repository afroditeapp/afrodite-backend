//! Common routes related to admin features

use axum::{Extension, extract::Query};
use http::StatusCode;
use tracing::error;

use manager_model::{
    BuildInfo, RebootQueryParam, SoftwareInfo, SoftwareOptionsQueryParam, SystemInfoList,
};
use model::{Account, AccountIdInternal};


use crate::api::{GetManagerApi, ReadDatabase, utils::Json};

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
    security(("api_key" = [])),
)]
pub async fn get_system_info<S: GetManagerApi + ReadDatabase>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SystemInfoList>, StatusCode> {
    let account = state
        .read_database()
        .account()
        .account(api_caller_account_id)
        .await
        .map_err(|e| {
            error!("get_system_info {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account.capablities().admin_server_maintentance_view_info {
        state
            .manager_api()
            .system_info()
            .await
            .map_err(|e| {
                error!("get_system_info {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(|data| data.into())
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
    security(("api_key" = [])),
)]
pub async fn get_software_info<S: GetManagerApi + ReadDatabase>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SoftwareInfo>, StatusCode> {
    let account = state
        .read_database()
        .account()
        .account(api_caller_account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account.capablities().admin_server_maintentance_view_info {
        state
            .manager_api()
            .software_info()
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(|data| data.into())
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
    security(("api_key" = [])),
)]
pub async fn get_latest_build_info<S: GetManagerApi + ReadDatabase>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<BuildInfo>, StatusCode> {
    let account: Account = state
        .read_database()
        .account()
        .account(api_caller_account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account.capablities().admin_server_maintentance_view_info {
        state
            .manager_api()
            .get_latest_build_info(software.software_options)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(|data| data.into())
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
    security(("api_key" = [])),
)]
pub async fn post_request_build_software<S: GetManagerApi + ReadDatabase>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<(), StatusCode> {
    let account: Account = state
        .read_database()
        .account()
        .account(api_caller_account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account
        .capablities()
        .admin_server_maintentance_update_software
    {
        state
            .manager_api()
            .request_build_software_from_build_server(software.software_options)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
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
    security(("api_key" = [])),
)]
pub async fn post_request_update_software<S: GetManagerApi + ReadDatabase>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Query(reboot): Query<RebootQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<(), StatusCode> {
    let account: Account = state
        .read_database()
        .account()
        .account(api_caller_account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account
        .capablities()
        .admin_server_maintentance_update_software
    {
        state
            .manager_api()
            .request_update_software(software.software_options, reboot.reboot)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
