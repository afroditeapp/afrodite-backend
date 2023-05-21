use std::net::SocketAddr;

use axum::{middleware::Next, response::Response, extract::ConnectInfo};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode, http::request, Method};

use jsonwebtoken::{DecodingKey, jwk::{Jwk, JwkSet}, Validation};
use serde::Deserialize;
use serde_json::Value;
use tracing::log::info;
use url::Url;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify,
};

use crate::server::internal::AuthResponse;

use super::{model::ApiKey, GetInternalApi, GetApiKeys};

pub const API_KEY_HEADER_STR: &str = "x-api-key";
pub static API_KEY_HEADER: header::HeaderName = header::HeaderName::from_static(API_KEY_HEADER_STR);

pub async fn authenticate_with_api_key<T, S: GetApiKeys>(
    state: S,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(API_KEY_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = ApiKey::new(key_str.to_string());

    if let Some(id) = state.api_keys().api_key_and_connection_exists(&key, addr).await {
        req.extensions_mut().insert(id);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub struct ApiKeyHeader(ApiKey);

impl ApiKeyHeader {
    pub fn key(&self) -> &ApiKey {
        &self.0
    }
}

impl Header for ApiKeyHeader {
    fn name() -> &'static headers::HeaderName {
        &API_KEY_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let value = value.to_str().map_err(|_| headers::Error::invalid())?;
        Ok(ApiKeyHeader(ApiKey::new(value.to_string())))
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        let header = HeaderValue::from_str(self.0.as_str()).unwrap();
        values.extend(std::iter::once(header))
    }
}

/// Utoipa API doc security config
pub struct SecurityApiTokenDefault;

impl Modify for SecurityApiTokenDefault {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(utoipa::openapi::security::ApiKey::Header(
                    ApiKeyValue::new(API_KEY_HEADER_STR),
                )),
            )
        }
    }
}

// pub async fn get_account<S: GetUsers, T>(
//     state: &S,
//     id: AccountIdLight,
//     fun: impl Fn(&Arc<AccountStateInRam>) -> T
// ) -> Result<T, StatusCode> {
//     state
//         .users()
//         .read()
//         .await
//         .get(&id)
//         .ok_or(StatusCode::UNAUTHORIZED)
//         .map(fun)
// }

// pub async fn get_account_from_api_key<S: GetApiKeys, T>(
//     state: &S,
//     id: &ApiKey,
//     fun: impl Fn(&Arc<AccountStateInRam>) -> T
// ) -> Result<T, StatusCode> {
//     state
//         .api_keys()
//         .read()
//         .await
//         .get(&id)
//         .ok_or(StatusCode::UNAUTHORIZED)
//         .map(fun)
// }


#[derive(Debug, Deserialize)]
struct GoogleInfo {
    iss: Option<String>,
    sub: Option<String>,
    jti: Option<String>,
    exp: Option<String>,
    iat: Option<String>,
    client_id: Option<String>,
    scope: Option<String>,
}

pub async fn validate_sign_in_with_google_token(token: String) -> Result<bool, ()> {
    // info!("{:?}", &token);
    // use base64::Engine;
    // let token = base64::engine::general_purpose::STANDARD_NO_PAD.decode(&token).unwrap();
    // let token =  String::from_utf8(token).unwrap();
    info!("{:?}", &token);

    let not_validated_header = jsonwebtoken::decode_header(&token).map_err(|_| ())?;
    info!("{:?}", &not_validated_header);
    let wanted_kid = not_validated_header.kid.unwrap();


    let x = reqwest::Request::new(Method::GET, Url::parse("https://www.googleapis.com/oauth2/v3/certs").unwrap());
    let c = reqwest::Client::new();
    let r = c.execute(x).await.unwrap();
    info!("{:?}", &r);
    let jwk_set: JwkSet = r.json().await.unwrap();
    info!("{:?}", &jwk_set);
    let jwk = jwk_set.find(&wanted_kid).unwrap();
    let key = &DecodingKey::from_jwk(&jwk).unwrap();
    info!("decoding key successfull");
    let v =  &Validation::new(jwk.common.algorithm.unwrap());
    info!("{:?}", &v);
    let d = jsonwebtoken::decode::<Value>(&token, key, v);
    match d {
        Ok(value) => {
            // TODO: check aud and azp. There is Validation struct in jsonwebtoken
            info!("{:?}", value);
            Ok(true)
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Ok(false)
        }
    }
}



pub async fn validate_sign_in_with_apple_token(token: String) -> Result<bool, ()> {
    info!("{:?}", &token);

    let not_validated_header = jsonwebtoken::decode_header(&token).map_err(|_| ())?;
    info!("{:?}", &not_validated_header);

    // TODO: validation

    Ok(true)
}
