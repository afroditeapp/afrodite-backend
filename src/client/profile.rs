
use std::{collections::HashMap, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use headers::Header;
use hyper::StatusCode;
use reqwest::{Client, Request, Url};
use tokio::sync::{Mutex, RwLock};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use error_stack::{Result, ResultExt, IntoReport, Context};

use crate::{
    api::{
        self,
        model::{ApiKey, AccountId, AccountIdLight, Profile},
        account::{internal::PATH_CHECK_API_KEY, PATH_REGISTER, PATH_LOGIN},
        utils::{
            ApiKeyHeader,
        },
        ApiDoc, GetApiKeys, GetRouterDatabaseHandle, GetSessionManager, GetUsers, ReadDatabase,
        WriteDatabase, profile::{PATH_GET_PROFILE, PATH_POST_PROFILE},
    },
    utils::IntoReportExt, server::internal::{},
};

use crate::server::{
    app::AppState,
    database::{read::ReadCommands, write::WriteCommands, RouterDatabaseHandle},
    session::{SessionManager, AccountStateInRam},
};

use super::{HttpRequestError, get_api_url, StatusCodeError};


// Public API


#[derive(Debug, Clone)]
pub enum ProfileApiRequest {
    GetProfile,
    PostProfile,
}

impl std::fmt::Display for ProfileApiRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Profile API request: {:?}", self))
    }
}

#[derive(Debug, Default, Clone)]
pub struct ProfileApiUrls {
    get_profile: Option<Url>,
    post_profile: Option<Url>,
}

impl ProfileApiUrls {
    pub fn new(base_url: Url) -> Result<Self, url::ParseError> {
        Ok(Self {
            get_profile: Some(base_url.join(PATH_GET_PROFILE)?),
            post_profile: Some(base_url.join(PATH_POST_PROFILE)?),
        })
    }
}


pub struct ProfileApi<'a> {
    client: &'a Client,
    urls: &'a ProfileApiUrls,
}

impl <'a> ProfileApi<'a> {
    pub fn new(client: &'a Client, urls: &'a ProfileApiUrls) -> Self {
        Self {
            client,
            urls,
        }
    }

    pub async fn get_profile(
        &self,
        api_key: ApiKey,
        profile_account_id: AccountId,
    ) -> Result<Profile, HttpRequestError> {
        let url = get_api_url(&self.urls.get_profile)?
            .join(profile_account_id.as_str())
            .into_error_with_info(
                HttpRequestError::ApiUrlJoinError,
                ProfileApiRequest::GetProfile,
            )?;

        let request = self
            .client
            .get(url)
            .header(ApiKeyHeader::name(), api_key.as_str())
            .build()
            .unwrap();

        let response = self.client.execute(request).await
            .into_error_with_info(
                HttpRequestError::Reqwest,
                ProfileApiRequest::GetProfile,
            )?;

        if response.status() == StatusCode::OK {
            let id: Profile = response.json().await.into_error_with_info(
                HttpRequestError::SerdeDeserialize,
                ProfileApiRequest::GetProfile,
            )?;
            Ok(id)
        } else {
            Err(StatusCodeError(response.status())).into_error_with_info(
                HttpRequestError::StatusCode,
                ProfileApiRequest::GetProfile,
            )
        }
    }

    pub async fn post_profile(&self, api_key: ApiKey, profile: Profile) -> Result<(), HttpRequestError> {
        let url = get_api_url(&self.urls.post_profile)?;

        let request = self
            .client
            .post(url)
            .header(ApiKeyHeader::name(), api_key.as_str())
            .json(&profile)
            .build()
            .unwrap();

        let response = self.client.execute(request).await
            .into_error_with_info(
                HttpRequestError::Reqwest,
                ProfileApiRequest::PostProfile,
            )?;
        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(StatusCodeError(response.status())).into_error_with_info(
                HttpRequestError::StatusCode,
                ProfileApiRequest::PostProfile,
            )
        }
    }
}
