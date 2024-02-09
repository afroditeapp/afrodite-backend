//! Routes for server to server connections

use api_internal::{Configuration, InternalApi};
use config::{Config, InternalApiUrls};
use hyper::StatusCode;
use model::{
    AccessToken, Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
    ProfileInternal,
};

use tracing::{error, info, warn};

use super::data::{read::ReadCommands, utils::AccessTokenManager};
use crate::{
    app::{GetAccessTokens, GetConfig, ReadData, WriteData},
    data::{WithInfo, WrappedWithInfo},
    result::{Result, WrappedContextExt, WrappedResultExt},
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

#[derive(thiserror::Error, Debug)]
pub enum InternalApiError {
    #[error("API request failed")]
    ApiRequest,

    #[error("Database call failed")]
    DataError,

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

    #[error("Required server component is not enabled")]
    MissingComponent,
}

// TOOD: What is PrintWarningsTriggersAtomics?
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
            .ok_or(InternalApiError::AccountApiUrlNotConfigured.report())
    }

    pub fn media(&self) -> Result<&Configuration, InternalApiError> {
        self.media
            .as_ref()
            .ok_or(InternalApiError::MediaApiUrlNotConfigured.report())
    }
}

pub enum AuthResponse {
    Ok,
    Unauthorized,
}

/// Handle requests to internal API. If the required feature is located
/// on the current server, then request is not made.
pub struct InternalApiManager<'a, S> {
    state: &'a S,
    api_client: &'a InternalApiClient,
}

impl<'a, S> InternalApiManager<'a, S> {
    pub fn new(state: &'a S, api_client: &'a InternalApiClient) -> Self {
        Self { state, api_client }
    }
}

impl<S: GetAccessTokens> InternalApiManager<'_, S> {
    fn access_tokens(&self) -> AccessTokenManager {
        self.state.access_tokens()
    }
}

impl<S: GetConfig + GetAccessTokens> InternalApiManager<'_, S> {
    /// Check that API key is valid. Use this only from AccessToken checker handler.
    /// This function will cache the account ID, so it can be found using normal
    /// database calls after this runs.
    pub async fn check_access_token(
        &self,
        key: AccessToken,
    ) -> Result<AuthResponse, InternalApiError> {
        if self
            .access_tokens()
            .access_token_exists(&key)
            .await
            .is_some()
        {
            Ok(AuthResponse::Ok)
        } else if !self.config().components().account {
            // Check AccessToken from external service

            let result = InternalApi::check_access_token(self.api_client.account()?, key).await;

            match result {
                Ok(_res) => {
                    // TODO: Cache this API key. Also needed for initializing
                    // database tables.
                    Ok(AuthResponse::Ok)
                }
                Err(api_internal::Error::ResponseError(response))
                    if response.status.as_u16() == StatusCode::UNAUTHORIZED.as_u16() =>
                {
                    // TODO: NOTE: Logging every error is not good as it would spam
                    // the log, but maybe an error counter or logging just
                    // once for a while.
                    Ok(AuthResponse::Unauthorized)
                }
                Err(e) => Err(e).change_context(InternalApiError::ApiRequest),
            }
        } else {
            Ok(AuthResponse::Unauthorized)
        }
    }
}

impl<S: GetConfig> InternalApiManager<'_, S> {
    fn config(&self) -> &Config {
        self.state.config()
    }
}
impl<S: GetAccessTokens + GetConfig + ReadData> InternalApiManager<'_, S> {
    pub async fn get_account_state(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Account, InternalApiError> {
        if self.config().components().account {
            self.read_database()
                .account()
                .account(account_id)
                .await
                .change_context(InternalApiError::DataError)
        } else {
            // TODO: Save account state to cache?

            let account =
                InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
                    .await
                    .change_context(InternalApiError::ApiRequest)?;

            Ok(account)
        }
    }
}

pub struct Data<'a> {
    pub capabilities: &'a mut Capabilities,
    pub state: &'a mut AccountState,
    pub is_profile_public: &'a mut bool,
}

impl<S: GetAccessTokens + GetConfig + ReadData + WriteData> InternalApiManager<'_, S> {
    /// Only account server can modify the state. Does nothing if the server
    /// does not have account component enabled.
    ///
    /// Returns the modified capabilities.
    pub async fn modify_and_sync_account_state(
        &self,
        account_id: AccountIdInternal,
        action: impl FnOnce(Data),
    ) -> Result<Capabilities, InternalApiError> {
        if !self.config().components().account {
            warn!("Account component not enabled, cannot modify account state");
            // TODO: Would it be better to return error here?
            return Err(InternalApiError::MissingComponent.report());
        }

        let mut current = self
            .read_database()
            .account()
            .account(account_id)
            .await
            .change_context(InternalApiError::DataError)?
            .into_capablities();

        let mut shared_state = self
            .read_database()
            .common()
            .shared_state(account_id)
            .await
            .change_context(InternalApiError::DataError)?;

        action(Data {
            capabilities: &mut current,
            state: &mut shared_state.account_state,
            is_profile_public: &mut shared_state.is_profile_public,
        });

        let modified_capabilities_copy = current.clone();
        let modified_shared_state_copy = shared_state.clone();
        self.state
            .write(move |cmds| async move {
                cmds.account()
                    .update_account_state_and_capabilities(
                        account_id,
                        Some(modified_shared_state_copy),
                        Some(modified_capabilities_copy),
                    )
                    .await
            })
            .await
            .change_context(InternalApiError::DataError)?;

        // TODO add sync account state command to common internal api

        if !self.config().components().profile {
            // let account =
            // InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
            //     .await
            //     .change_context(InternalApiError::ApiRequest)?;
        }

        if !self.config().components().media {
            // let account =
            // InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
            //     .await
            //     .change_context(InternalApiError::ApiRequest)?;
        }

        if !self.config().components().chat {
            // let account =
            // InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
            //     .await
            //     .change_context(InternalApiError::ApiRequest)?;
        }

        Ok(current)
    }
}

impl<S: ReadData> InternalApiManager<'_, S> {
    fn read_database(&self) -> ReadCommands {
        self.state.read()
    }
}

impl<S: GetAccessTokens + GetConfig + ReadData> InternalApiManager<'_, S> {
    pub async fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), InternalApiError> {
        if self.config().components().media {
            let _request = self
                .read_database()
                .moderation_request(account_id)
                .await
                .change_context(InternalApiError::DataError)?
                .ok_or(InternalApiError::MissingValue)
                .with_info(account_id)?;

            // if request.content.initial_moderation_security_image.is_some() {
            //     Ok(())
            // } else {
            Err(InternalApiError::MissingValue).with_info(account_id)
            // }
        } else {
            InternalApi::media_check_moderation_request_for_account(
                self.api_client.media()?,
                account_id.as_id(),
            )
            .await
            .change_context(InternalApiError::MissingValue)
        }
    }
}

impl<S: GetAccessTokens + GetConfig + ReadData + WriteData> InternalApiManager<'_, S> {
    pub async fn profile_initial_setup(
        // TODO
        &self,
        account_id: AccountIdInternal,
        profile_name: String,
    ) -> Result<(), InternalApiError> {
        if self.config().components().profile {
            self.state
                .write(move |cmds| async move {
                    cmds.profile().profile_name(account_id, profile_name).await
                })
                .await
                .change_context(InternalApiError::DataError)
        } else {
            // TODO: Add method to internal profile API which will do the
            // initial setup (setting the name field) for user profile.
            Ok(())
        }
    }
}

impl<S: GetAccessTokens + GetConfig + ReadData + WriteData> InternalApiManager<'_, S> {
    /// Profile visiblity is set first to the profile server and in addition
    /// to changing the visibility the current proifle is returned (used for
    /// changing visibility for media server).
    pub async fn profile_api_set_profile_visiblity(
        &self,
        account_id: AccountIdInternal,
        boolean_setting: BooleanSetting,
    ) -> Result<(), InternalApiError> {
        if self.config().components().profile {
            self.state
                .write(move |data| async move {
                    data.profile()
                        .profile_update_visibility(
                            account_id,
                            boolean_setting.value,
                            false, // False overrides updates
                        )
                        .await
                })
                .await
                .change_context(InternalApiError::DataError)?;

            let profile: ProfileInternal = self
                .read_database()
                .profile()
                .profile(account_id)
                .await
                .change_context(InternalApiError::DataError)?;

            self.media_api_profile_visiblity(account_id, boolean_setting, profile.into())
                .await
                .change_context(InternalApiError::ApiRequest)?;

            Ok(())
        } else {
            // TODO: Request internal profile api
            todo!()
        }
    }
}

impl<S: GetConfig> InternalApiManager<'_, S> {
    pub async fn media_api_profile_visiblity(
        &self,
        _account_id: AccountIdInternal,
        _boolean_setting: BooleanSetting,
        _current_profile: Profile,
    ) -> Result<(), InternalApiError> {
        if self.config().components().media {
            // TODO: Save visibility information to cache?
            Ok(())
        } else {
            // TODO: request to internal media API
            Ok(())
        }
    }

    // TODO: Prevent creating a new moderation request when there is camera
    // image in the current one. Or also make possible to change the ongoing
    // moderation request but leave the camera image. Where information about
    // the camera image should be stored?
}
