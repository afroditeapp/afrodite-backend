//! Routes for server to server connections

use api_client::apis::{accountinternal_api, configuration::Configuration, mediainternal_api};
use axum::{
    routing::{get, post},
    Router,
};

use error_stack::{Result, ResultExt};

use hyper::StatusCode;

use tokio::sync::{Mutex, MutexGuard};
use tracing::{error, info};

use crate::{
    api::{
        self,
        model::{
            Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
            ProfileInternal,
        },
        GetConfig,
    },
    config::InternalApiUrls,
    utils::IntoReportExt,
};

use crate::{api::model::ApiKey, config::Config};

use super::{
    app::AppState,
    data::{
        commands::WriteCommandRunnerHandle,
        read::ReadCommands,
        utils::{AccountIdManager, ApiKeyManager}, SyncWriteHandle,
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

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_account_server_router(state: AppState) -> Router {
        let mut router = Router::new()
            .route(
                api::account::internal::PATH_INTERNAL_CHECK_API_KEY,
                get({
                    let state = state.clone();
                    move |body| api::account::internal::check_api_key(body, state)
                }),
            )
            .route(
                api::account::internal::PATH_INTERNAL_GET_ACCOUNT_STATE,
                get({
                    let state = state.clone();
                    move |param1| api::account::internal::internal_get_account_state(param1, state)
                }),
            );

        if state.config().internal_api_config().bot_login {
            router = router
                .route(
                    api::account::PATH_REGISTER,
                    post({
                        let state = state.clone();
                        move || api::account::post_register(state)
                    }),
                )
                .route(
                    api::account::PATH_LOGIN,
                    post({
                        let state = state.clone();
                        move |body| api::account::post_login(body, state)
                    }),
                )
        }

        router
    }

    pub fn create_profile_server_router(state: AppState) -> Router {
        Router::new().route(
            api::profile::internal::PATH_INTERNAL_POST_UPDATE_PROFILE_VISIBLITY,
            post({
                let state = state.clone();
                move |p1, p2| {
                    api::profile::internal::internal_post_update_profile_visibility(p1, p2, state)
                }
            }),
        )
    }

    pub fn create_media_server_router(state: AppState) -> Router {
        Router::new()
            .route(
                api::media::internal::PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT,
                post({
                    let state = state.clone();
                    move |parameter1| {
                        api::media::internal::internal_get_check_moderation_request_for_account(
                            parameter1, state,
                        )
                    }
                }),
            )
            .route(
                api::media::internal::PATH_INTERNAL_POST_UPDATE_PROFILE_IMAGE_VISIBLITY,
                post({
                    let state = state.clone();
                    move |p1, p2, p3| {
                        api::media::internal::internal_post_update_profile_image_visibility(
                            p1, p2, p3, state,
                        )
                    }
                }),
            )
    }

    pub fn create_chat_server_router(_state: AppState) -> Router {
        Router::new()
        // .route(
        //     api::media::internal::PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT,
        //     post({
        //         let state = state.clone();
        //         move |parameter1| {
        //             api::media::internal::internal_get_check_moderation_request_for_account(
        //                 parameter1, state,
        //             )
        //         }
        //     }),
        // )
    }
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
    write_database: &'a WriteCommandRunnerHandle,
    account_id_manager: AccountIdManager<'a>,
    write_mutex: &'a Mutex<SyncWriteHandle>,
}

impl<'a> InternalApiManager<'a> {
    pub fn new(
        config: &'a Config,
        api_client: &'a InternalApiClient,
        keys: ApiKeyManager<'a>,
        read_database: ReadCommands<'a>,
        write_database: &'a WriteCommandRunnerHandle,
        account_id_manager: AccountIdManager<'a>,
        write_mutex: &'a Mutex<SyncWriteHandle>,
    ) -> Self {
        Self {
            config,
            api_client,
            keys,
            read_database,
            write_database,
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

            let result = accountinternal_api::check_api_key(
                self.api_client.account()?,
                api_client::models::ApiKey {
                    api_key: key.into_string(),
                },
            )
            .await;

            match result {
                Ok(_res) => {
                    // TODO: Cache this API key. Also needed for initializing
                    // database tables.
                    Ok(AuthResponse::Ok)
                }
                Err(api_client::apis::Error::ResponseError(response))
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

            let account = accountinternal_api::internal_get_account_state(
                self.api_client.account()?,
                &account_id.as_light().to_string(),
            )
            .await
            .into_error(InternalApiError::ApiRequest)?;

            let state = match account.state {
                api_client::models::AccountState::InitialSetup => AccountState::InitialSetup,
                api_client::models::AccountState::Normal => AccountState::Normal,
                api_client::models::AccountState::Banned => AccountState::Banned,
                api_client::models::AccountState::PendingDeletion => AccountState::PendingDeletion,
            };

            macro_rules! copy_capablities {
                ($account:expr,  $( $name:ident , )* ) => {
                    Capabilities {
                        $( $name: $account.capablities.$name.unwrap_or(false), )*
                        ..Capabilities::default()
                    }
                };
            }
            // TODO: Add missing capabilities
            let capabilities = copy_capablities!(
                account,
                admin_modify_capablities,
                admin_setup_possible,
                admin_moderate_profiles,
                admin_moderate_images,
                admin_view_all_profiles,
                admin_view_private_info,
                admin_view_profile_history,
                admin_ban_profile,
                banned_edit_profile,
                view_public_profiles,
            );

            Ok(Account::new_from(state, capabilities))
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
            mediainternal_api::internal_get_check_moderation_request_for_account(
                self.api_client.media()?,
                &account_id.as_light().to_string(),
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
