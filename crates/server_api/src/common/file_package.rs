use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State},
    response::Html,
};
use axum_extra::TypedHeader;
use headers::{
    CacheControl, ContentEncoding, ContentType, ETag, Header, HeaderName, HeaderValue, IfNoneMatch,
};
use http::StatusCode;
use server_data::app::GetConfig;
use simple_backend::{
    app::{FilePackageProvider, MaxMindDbDataProvider},
    create_counters,
};
use simple_backend_config::file::IpAddressAccessConfig;

use crate::{S, utils::IfNoneMatchExtensions};

#[derive(Debug, Clone)]
pub struct ServiceWorkerAllowed(HeaderValue);

impl Header for ServiceWorkerAllowed {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("service-worker-allowed");
        &NAME
    }

    fn decode<'i, I>(_values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        // Not needed for response-only header
        Err(headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(self.0.clone()));
    }
}

impl ServiceWorkerAllowed {
    pub fn root_scope() -> Self {
        Self(HeaderValue::from_static("/"))
    }
}

type StaticFileResponse = (
    TypedHeader<ETag>,
    TypedHeader<CacheControl>,
    TypedHeader<ContentType>,
    Option<TypedHeader<ContentEncoding>>,
    Option<TypedHeader<ServiceWorkerAllowed>>,
    Bytes,
);

pub const PATH_FILE_PACKAGE_ACCESS: &str = "/app/{*path}";

pub async fn get_file_package_access(
    State(state): State<S>,
    Path(path_parts): Path<Vec<String>>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, Html<String>)> {
    COMMON.get_file_package_access.incr();

    check_ip_allowlist(&state, address).await?;

    let wanted_file = path_parts.join("/");
    let file = state
        .file_package()
        .static_file(&wanted_file)
        .ok_or((StatusCode::NOT_FOUND, empty_html()))?;

    if browser_etag.matches(state.etag_utils().immutable_content()) {
        return Err((StatusCode::NOT_MODIFIED, empty_html()));
    }

    const MONTH_SECONDS: u64 = 60 * 60 * 24 * 30;
    let cache_control = CacheControl::new()
        .with_max_age(Duration::from_secs(MONTH_SECONDS * 12))
        .with_must_revalidate()
        .with_public()
        .with_immutable();

    let service_worker_header = if wanted_file.ends_with("/sw.js") {
        Some(TypedHeader(ServiceWorkerAllowed::root_scope()))
    } else {
        None
    };

    Ok((
        TypedHeader(state.etag_utils().immutable_content().clone()),
        TypedHeader(cache_control),
        TypedHeader(file.content_type),
        file.content_encoding.map(TypedHeader),
        service_worker_header,
        file.data,
    ))
}

pub const PATH_FILE_PACKAGE_ACCESS_ROOT: &str = "/";

pub async fn get_file_package_access_root(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, Html<String>)> {
    COMMON.get_file_package_access_root.incr();
    return_index_html(state, address, browser_etag).await
}

pub const PATH_FILE_PACKAGE_ACCESS_PWA_INDEX_HTML: &str = "/app/index.html";

pub async fn get_file_package_access_pwa_index_html(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, Html<String>)> {
    COMMON.get_file_package_access_pwa_index_html.incr();
    return_index_html(state, address, browser_etag).await
}

async fn return_index_html(
    state: S,
    address: SocketAddr,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, Html<String>)> {
    check_ip_allowlist(&state, address).await?;

    if browser_etag.matches(state.etag_utils().server_start_time()) {
        return Err((StatusCode::NOT_MODIFIED, empty_html()));
    }

    let file = state
        .file_package()
        .index_html()
        .ok_or((StatusCode::NOT_FOUND, empty_html()))?;

    let cache_control = CacheControl::new().with_no_cache();

    Ok((
        TypedHeader(state.etag_utils().server_start_time().clone()),
        TypedHeader(cache_control),
        TypedHeader(file.content_type),
        file.content_encoding.map(TypedHeader),
        None,
        file.data,
    ))
}

async fn check_ip_allowlist(
    state: &S,
    address: SocketAddr,
) -> Result<(), (StatusCode, Html<String>)> {
    if let Some(config) = state.config().simple_backend().file_package() {
        if is_ip_address_accepted(state, address, &config.acccess).await {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, create_access_denied_html(address)))
        }
    } else {
        Err((StatusCode::NOT_FOUND, empty_html()))
    }
}

pub async fn is_ip_address_accepted(
    state: &S,
    address: SocketAddr,
    config: &IpAddressAccessConfig,
) -> bool {
    if config.allow_all_ip_addresses || config.ip_allowlist.iter().any(|v| *v == address.ip()) {
        return true;
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
                    return true;
                }
            }
        }
    }

    false
}

fn empty_html() -> Html<String> {
    Html(String::new())
}

fn create_access_denied_html(ip_address: SocketAddr) -> Html<String> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Access Denied</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
        }}
        .container {{
            background: white;
            padding: 3rem;
            border-radius: 1rem;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            max-width: 500px;
            text-align: center;
        }}
        .icon {{
            font-size: 4rem;
            margin-bottom: 1rem;
        }}
        h1 {{
            margin: 0 0 1rem 0;
            font-size: 2rem;
            color: #d32f2f;
        }}
        p {{
            margin: 0.5rem 0;
            color: #666;
            line-height: 1.6;
        }}
        .ip-address {{
            font-family: 'Courier New', monospace;
            background: #f5f5f5;
            padding: 0.5rem 1rem;
            border-radius: 0.5rem;
            margin: 1.5rem 0;
            font-weight: bold;
            color: #333;
        }}
        .footer {{
            margin-top: 2rem;
            font-size: 0.875rem;
            color: #999;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">🚫</div>
        <h1>Access Denied</h1>
        <p>Sorry, access to this application is not allowed from your current IP address.</p>
        <div class="ip-address">Your IP: {}</div>
        <p>If you believe this is an error, please contact the system administrator.</p>
        <div class="footer">Error Code: 403 Forbidden</div>
    </div>
</body>
</html>"#,
        ip_address.ip()
    );
    Html(html)
}

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_FILE_PACKAGE_COUNTERS_LIST,
    get_file_package_access,
    get_file_package_access_root,
    get_file_package_access_pwa_index_html,
);
