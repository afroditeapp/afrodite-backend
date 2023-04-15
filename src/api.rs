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
        database::{current::read::SqliteReadCommands, write::WriteCommands, RouterDatabaseHandle, read::ReadCommands, utils::{ApiKeyManager, AccountIdManager}},
        session::{SessionManager},
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
        account::post_complete_setup,
        account::post_delete,
        account::put_setting_profile_visiblity,
        account::get_account_state,
        account::get_deletion_status,
        account::delete_cancel_deletion,
        account::internal::check_api_key,
        profile::get_profile,
        profile::get_default_profile,
        profile::get_next_profile_page,
        profile::post_profile,
        profile::post_reset_profile_paging,
        profile::put_location,
        media::get_image,
        media::get_moderation_request,
        media::put_moderation_request,
        media::put_image_to_moderation_slot,
        media::post_handle_moderation_request,
        media::patch_moderation_request_list,
        media::internal::post_image,
    ),
    components(schemas(
        account::data::AccountId,
        account::data::AccountIdLight,
        account::data::ApiKey,
        account::data::Account,
        account::data::AccountState,
        account::data::AccountSetup,
        account::data::Capabilities,
        account::data::BooleanSetting,
        account::data::DeleteStatus,
        profile::data::Profile,
        profile::data::ProfilePage,
        profile::data::ProfileLink,
        profile::data::Location,
        media::data::ImageFileName,
        media::data::ImageFile,
        media::data::NewModerationRequest,
        media::data::ModerationRequest,
        media::data::ModerationList,
        media::data::HandleModerationRequest,
        media::data::SlotId,
        media::data::ContentId,
    )),
    modifiers(&SecurityApiTokenDefault),
    info(
        title = "pihka-backend",
        description = "Pihka backend API",
        version = "0.1.0"
    )
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
    fn api_keys(&self) -> ApiKeyManager<'_>;
}

pub trait GetUsers {
    /// All users registered in the service.
    fn users(&self) -> AccountIdManager<'_>;
}


pub trait WriteDatabase {
    fn write_database(
        &self,
        lock_id: AccountIdLight,
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
