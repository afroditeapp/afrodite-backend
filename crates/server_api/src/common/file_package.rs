use std::{net::SocketAddr, time::Duration};

use axum::{
    body::{Body, Bytes},
    extract::{ConnectInfo, Path, State},
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
pub struct AcceptLanguage(String);

impl Header for AcceptLanguage {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("accept-language");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let value_str = value.to_str().map_err(|_| headers::Error::invalid())?;

        // Parse the first language from Accept-Language header
        // Format: "en-US,en;q=0.9,fi;q=0.8" -> extract "en-US" or "en"
        let first_lang = value_str
            .split(',')
            .next()
            .and_then(|lang| lang.split(';').next())
            .map(|lang| lang.trim())
            .unwrap_or("");

        // Extract just the language code (e.g., "en" from "en-US")
        let lang_code = first_lang.split('-').next().unwrap_or(first_lang);

        Ok(AcceptLanguage(lang_code.to_string()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        if let Ok(value) = HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(value));
        }
    }
}

impl AcceptLanguage {
    pub fn language(&self) -> &str {
        &self.0
    }
}

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
    accept_language: Option<TypedHeader<AcceptLanguage>>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, TypedHeader<ContentType>, Body)> {
    COMMON.get_file_package_access.incr();

    check_ip_allowlist(&state, address, accept_language).await?;

    let wanted_file = path_parts.join("/");
    let file = state.file_package().static_file(&wanted_file).ok_or((
        StatusCode::NOT_FOUND,
        TypedHeader(ContentType::html()),
        Body::empty(),
    ))?;

    if browser_etag.matches(state.etag_utils().immutable_content()) {
        return Err((
            StatusCode::NOT_MODIFIED,
            TypedHeader(ContentType::html()),
            Body::empty(),
        ));
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
    accept_language: Option<TypedHeader<AcceptLanguage>>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, TypedHeader<ContentType>, Body)> {
    COMMON.get_file_package_access_root.incr();
    return_index_html(state, address, accept_language, browser_etag).await
}

pub const PATH_FILE_PACKAGE_ACCESS_PWA_INDEX_HTML: &str = "/app/index.html";

pub async fn get_file_package_access_pwa_index_html(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, TypedHeader<ContentType>, Body)> {
    COMMON.get_file_package_access_pwa_index_html.incr();
    return_index_html(state, address, accept_language, browser_etag).await
}

async fn return_index_html(
    state: S,
    address: SocketAddr,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<StaticFileResponse, (StatusCode, TypedHeader<ContentType>, Body)> {
    check_ip_allowlist(&state, address, accept_language).await?;

    if browser_etag.matches(state.etag_utils().server_start_time()) {
        return Err((
            StatusCode::NOT_MODIFIED,
            TypedHeader(ContentType::html()),
            Body::empty(),
        ));
    }

    let file = state.file_package().index_html().ok_or((
        StatusCode::NOT_FOUND,
        TypedHeader(ContentType::html()),
        Body::empty(),
    ))?;

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
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> Result<(), (StatusCode, TypedHeader<ContentType>, Body)> {
    if let Some(config) = state.config().simple_backend().file_package() {
        if is_ip_address_accepted(state, address, &config.acccess).await {
            Ok(())
        } else {
            Err(create_access_denied_response(
                state,
                address,
                accept_language,
            ))
        }
    } else {
        Err((
            StatusCode::NOT_FOUND,
            TypedHeader(ContentType::html()),
            Body::empty(),
        ))
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

fn create_access_denied_response(
    state: &S,
    ip_address: SocketAddr,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> (StatusCode, TypedHeader<ContentType>, Body) {
    let web_config = state.config().web_content();
    let language = accept_language.as_ref().map(|h| h.language());
    let page = web_config
        .get(language.as_ref())
        .access_denied(&ip_address.ip().to_string());

    match page {
        Ok(page) => {
            let content_type = if page.is_html {
                ContentType::html()
            } else {
                ContentType::text_utf8()
            };
            (
                StatusCode::FORBIDDEN,
                TypedHeader(content_type),
                Body::from(page.content),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            TypedHeader(ContentType::text_utf8()),
            Body::from("Internal Server Error"),
        ),
    }
}

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_FILE_PACKAGE_COUNTERS_LIST,
    get_file_package_access,
    get_file_package_access_root,
    get_file_package_access_pwa_index_html,
);
