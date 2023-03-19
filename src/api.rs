//! HTTP API types and request handlers for all servers.

// Routes
pub mod account;
pub mod media;
pub mod profile;

pub mod model;
pub mod utils;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};
use utoipa::{Modify, OpenApi};

use crate::{
    client::{account::AccountInternalApi, media::MediaInternalApi},
    config::Config,
    server::{
        database::{current::read::SqliteReadCommands, write::WriteCommands, RouterDatabaseHandle, read::ReadCommands},
        session::{AccountStateInRam, SessionManager},
    },
};

use self::model::{AccountIdInternal, ApiKey, AccountIdLight};

use utils::SecurityApiTokenDefault;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        account::post_register,
        account::post_login,
        account::post_account_setup,
        account::get_account_state,
        account::internal::check_api_key,
        profile::get_profile,
        profile::get_default_profile,
        profile::post_profile,
        media::get_image,
        media::internal::post_image,
    ),
    components(schemas(
        account::data::AccountId,
        account::data::AccountIdInternal,
        account::data::ApiKey,
        account::data::Account,
        account::data::AccountState,
        account::data::AccountSetup,
        account::data::Capabilities,
        profile::data::Profile,
        media::data::ImageFileName,
        media::data::ImageFile,
    )),
    modifiers(&SecurityApiTokenDefault),
)]
pub struct ApiDoc;

// App state getters

pub trait GetSessionManager {
    fn session_manager(&self) -> &SessionManager;
}

pub trait GetRouterDatabaseHandle {
    fn database(&self) -> &RouterDatabaseHandle;
}

pub trait GetApiKeys {
    /// Users which are logged in.
    fn api_keys(&self) -> &RwLock<HashMap<ApiKey, Arc<AccountStateInRam>>>;
}

pub trait GetUsers {
    /// All users registered in the service.
    fn users(&self) -> &RwLock<HashMap<AccountIdLight, Arc<AccountStateInRam>>>;
}


pub trait WriteDatabase {
    fn write_database(
        &self,
    ) -> WriteCommands<'_>;
}

pub trait ReadDatabase {
    fn read_database(&self) -> ReadCommands<'_>;
}

pub trait GetCoreServerInternalApi {
    fn core_server_internal_api(&self) -> AccountInternalApi;
}

pub trait GetMediaServerInternalApi {
    fn media_server_internal_api(&self) -> MediaInternalApi;
}

pub trait GetConfig {
    fn config(&self) -> &Config;
}
