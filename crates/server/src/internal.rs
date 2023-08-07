//! Routes for server to server connections

use api_internal::{Configuration, InternalApi};
use axum::{
    routing::{get, post},
    Router,
};

use error_stack::{Result, ResultExt};

use hyper::StatusCode;

use tokio::sync::{Mutex, MutexGuard};
use tracing::{error, info};

use model::{
    Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
    ProfileInternal, ApiKey,
};
use utils::IntoReportExt;
use crate::{
    api::{
        self,
        GetConfig,
    },

};
use config::InternalApiUrls;
use config::{Config};

use super::{
    app::AppState,
    data::{
        read::ReadCommands,
        utils::{AccountIdManager, ApiKeyManager},
        SyncWriteHandle,
    },
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

#[derive(thiserror::Error, Debug)]
pub enum InternalApiError {
    #[error("API request failed")]
    ApiRequest,

    #[error("Database call failed")]
    DatabaseError,

    #[error("Account API URL not configured")]
    AccountApiUrlNotConfigured,

    #[error("Media API URL not configured")]
    MediaApiUrlNotConfigured,
    // #[error("Wrong status code")]
    // StatusCode,

    // #[error("Joining text to URL failed")]
    // ApiUrlJoinError,
    #[error("Missing value")]
    MissingValue,

    #[error("Invalid value")]
    InvalidValue,
}

// TOOD: PrintWarningsTriggersAtomics?
pub struct PrintWarningsTriggersAtomics {}

pub struct InternalApiClient {
    account: Option<Configuration>,
    media: Option<Configuration>,
}

impl InternalApiClient {
    pub fn new(base_urls: InternalApiUrls) -> Self {
        let client = reqwest::Client::new();

        let account = base_urls.account_base_url.map(|url| {
            let url = url.as_str().trim_end_matches('/').to_string();

            info!("Account internal API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                ..Configuration::default()
            }
        });

        let media = base_urls.media_base_url.map(|url| {
            let url = url.as_str().trim_end_matches('/').to_string();

            info!("Media internal API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                ..Configuration::default()
            }
        });

        Self { account, media }
    }

    pub fn account(&self) -> Result<&Configuration, InternalApiError> {
        self.account
            .as_ref()
            .ok_or(InternalApiError::AccountApiUrlNotConfigured.into())
    }

    pub fn media(&self) -> Result<&Configuration, InternalApiError> {
        self.media
            .as_ref()
            .ok_or(InternalApiError::MediaApiUrlNotConfigured.into())
    }
}

pub enum AuthResponse {
    Ok,
    Unauthorized,
}

/// Handle requests to internal API. If the required feature is located
/// on the current server, then request is not made.
pub struct InternalApiManager<'a> {
    config: &'a Config,
    api_client: &'a InternalApiClient,
    keys: ApiKeyManager<'a>,
    read_database: ReadCommands<'a>,
    account_id_manager: AccountIdManager<'a>,
    write_mutex: &'a Mutex<SyncWriteHandle>,
}

impl<'a> InternalApiManager<'a> {
    pub fn new(
        config: &'a Config,
        api_client: &'a InternalApiClient,
        keys: ApiKeyManager<'a>,
        read_database: ReadCommands<'a>,
        account_id_manager: AccountIdManager<'a>,
        write_mutex: &'a Mutex<SyncWriteHandle>,
    ) -> Self {
        Self {
            config,
            api_client,
            keys,
            read_database,
            account_id_manager,
            write_mutex,
        }
    }

    /// Check that API key is valid. Use this only from ApiKey checker handler.
    /// This function will cache the account ID, so it can be found using normal
    /// database calls after this runs.
    pub async fn check_api_key(&self, key: ApiKey) -> Result<AuthResponse, InternalApiError> {
        if self.keys.api_key_exists(&key).await.is_some() {
            Ok(AuthResponse::Ok)
        } else if !self.config.components().account {
            // Check ApiKey from external service

            let result = InternalApi::check_api_key(
                self.api_client.account()?,
                key,
            )
            .await;

            match result {
                Ok(_res) => {
                    // TODO: Cache this API key. Also needed for initializing
                    // database tables.
                    Ok(AuthResponse::Ok)
                }
                Err(api_internal::Error::ResponseError(response))
                    if response.status == StatusCode::UNAUTHORIZED =>
                {
                    // TODO: NOTE: Logging every error is not good as it would spam
                    // the log, but maybe an error counter or logging just
                    // once for a while.
                    Ok(AuthResponse::Unauthorized)
                }
                Err(e) => Err(e).into_error(InternalApiError::ApiRequest),
            }
        } else {
            Ok(AuthResponse::Unauthorized)
        }
    }

    pub async fn get_account_state(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Account, InternalApiError> {
        if self.config.components().account {
            self.read_database
                .read_json::<Account>(account_id)
                .await
                .change_context(InternalApiError::DatabaseError)
        } else {
            // TODO: Save account state to cache?

            let account = InternalApi::get_account_state(
                self.api_client.account()?,
                account_id.as_light(),
            )
            .await
            .into_error(InternalApiError::ApiRequest)?;

            Ok(account)
        }
    }

    pub async fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), InternalApiError> {
        if self.config.components().media {
            let request = self
                .read_database
                .moderation_request(account_id)
                .await
                .change_context(InternalApiError::DatabaseError)?
                .ok_or(InternalApiError::MissingValue)?;

            if request.content.slot_1_is_security_image() {
                Ok(())
            } else {
                Err(InternalApiError::MissingValue.into())
            }
        } else {
            InternalApi::media_check_moderation_request_for_account(
                self.api_client.media()?,
                account_id.as_light(),
            )
            .await
            .into_error(InternalApiError::MissingValue)
        }
    }

    /// Profile visiblity is set first to the profile server and in addition
    /// to changing the visibility the current proifle is returned (used for
    /// changing visibility for media server).
    pub async fn profile_api_set_profile_visiblity(
        &self,
        account_id: AccountIdInternal,
        boolean_setting: BooleanSetting,
    ) -> Result<(), InternalApiError> {
        if self.config.components().profile {
            self.get_write()
                .await
                .profile()
                .profile_update_visibility(
                    account_id,
                    boolean_setting.value,
                    false, // False overrides updates
                )
                .await
                .change_context(InternalApiError::DatabaseError)?;

            let profile: ProfileInternal = self
                .read_database
                .read_json(account_id)
                .await
                .change_context(InternalApiError::DatabaseError)?;

            self.media_api_profile_visiblity(account_id, boolean_setting, profile.into())
                .await
                .change_context(InternalApiError::ApiRequest)?;

            Ok(())
        } else {
            // TODO: Request internal profile api
            todo!()
        }
    }

    pub async fn media_api_profile_visiblity(
        &self,
        _account_id: AccountIdInternal,
        _boolean_setting: BooleanSetting,
        _current_profile: Profile,
    ) -> Result<(), InternalApiError> {
        if self.config.components().media {
            // TODO: Save visibility information to cache?
            Ok(())
        } else {
            // TODO: request to internal media API
            Ok(())
        }
    }

    pub async fn get_write(&self) -> MutexGuard<SyncWriteHandle> {
        self.write_mutex.lock().await
    }

    // TODO: Prevent creating a new moderation request when there is camera
    // image in the current one. Or also make possible to change the ongoing
    // moderation request but leave the camera image. Where information about
    // the camera image should be stored?
}
