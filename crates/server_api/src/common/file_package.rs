use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State},
};
use axum_extra::TypedHeader;
use headers::{ContentEncoding, ContentType};
use server_data::app::GetConfig;
use simple_backend::{
    app::{FilePackageProvider, MaxMindDbDataProvider},
    create_counters,
};
use simple_backend_config::file::IpAddressAccessConfig;

use crate::{S, utils::StatusCode};

// TODO(web): HTTP cache header support for file package access

pub const PATH_FILE_PACKAGE_ACCESS: &str = "/app/{*path}";

pub async fn get_file_package_access(
    State(state): State<S>,
    Path(path_parts): Path<Vec<String>>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Result<
    (
        TypedHeader<ContentType>,
        Option<TypedHeader<ContentEncoding>>,
        Bytes,
    ),
    StatusCode,
> {
    COMMON.get_file_package_access.incr();
    check_ip_allowlist(&state, address).await?;
    let wanted_file = path_parts.join("/");
    let file = state
        .file_package()
        .static_file(&wanted_file)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok((
        TypedHeader(file.content_type),
        file.content_encoding.map(TypedHeader),
        file.data,
    ))
}

pub const PATH_FILE_PACKAGE_ACCESS_ROOT: &str = "/";

pub async fn get_file_package_access_root(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Result<
    (
        TypedHeader<ContentType>,
        Option<TypedHeader<ContentEncoding>>,
        Bytes,
    ),
    StatusCode,
> {
    COMMON.get_file_package_access_root.incr();
    check_ip_allowlist(&state, address).await?;
    let file = state
        .file_package()
        .index_html()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok((
        TypedHeader(file.content_type),
        file.content_encoding.map(TypedHeader),
        file.data,
    ))
}

pub const PATH_FILE_PACKAGE_ACCESS_PWA_INDEX_HTML: &str = "/app/index.html";

pub async fn get_file_package_access_pwa_index_html(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Result<
    (
        TypedHeader<ContentType>,
        Option<TypedHeader<ContentEncoding>>,
        Bytes,
    ),
    StatusCode,
> {
    COMMON.get_file_package_access_pwa_index_html.incr();
    check_ip_allowlist(&state, address).await?;
    let file = state
        .file_package()
        .index_html()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok((
        TypedHeader(file.content_type),
        file.content_encoding.map(TypedHeader),
        file.data,
    ))
}

async fn check_ip_allowlist(state: &S, address: SocketAddr) -> Result<(), StatusCode> {
    if let Some(config) = state.config().simple_backend().file_package() {
        is_ip_address_accepted(state, address, &config.acccess).await
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn is_ip_address_accepted(
    state: &S,
    address: SocketAddr,
    config: &IpAddressAccessConfig,
) -> Result<(), StatusCode> {
    if config.allow_all_ip_addresses || config.ip_allowlist.iter().any(|v| *v == address.ip()) {
        return Ok(());
    }

    if !config.ip_country_allowlist.is_empty() {
        let ip_db = state.maxmind_db().current_db_ref().await;
        if let Some(ip_db) = ip_db.as_ref() {
            if let Some(country) = ip_db.get_country_ref(address.ip()) {
                if config
                    .ip_country_allowlist
                    .iter()
                    .any(|v| v == country.as_str())
                {
                    return Ok(());
                }
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_FILE_PACKAGE_COUNTERS_LIST,
    get_file_package_access,
    get_file_package_access_root,
    get_file_package_access_pwa_index_html,
);
