//! HTTP API types and request handlers for all servers.

// Routes
pub mod account;
pub mod common;
pub mod media;
pub mod profile;

pub mod model;
pub mod utils;

use utoipa::{Modify, OpenApi};

use crate::{
    config::Config,
    server::{
        app::sign_in_with::SignInWithManager,
        database::{
            commands::WriteCommandRunnerHandle,
            read::ReadCommands,
            utils::{AccountIdManager, ApiKeyManager},
        },
        internal::InternalApiManager,
    },
};

use utils::SecurityApiTokenDefault;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        common::get_connect_websocket,
        account::post_register,
        account::post_login,
        account::post_sign_in_with_login,
        account::post_account_setup,
        account::post_complete_setup,
        account::post_delete,
        account::put_setting_profile_visiblity,
        account::get_account_state,
        account::get_deletion_status,
        account::delete_cancel_deletion,
        account::internal::check_api_key,
        account::internal::internal_get_account_state,
        profile::get_profile,
        profile::post_get_next_profile_page,
        profile::post_profile,
        profile::post_reset_profile_paging,
        profile::put_location,
        profile::internal::internal_post_update_profile_visibility,
        media::get_primary_image_info,
        media::get_security_image_info,
        media::get_all_normal_images,
        media::get_image,
        media::get_moderation_request,
        media::put_moderation_request,
        media::put_image_to_moderation_slot,
        media::put_primary_image,
        media::post_handle_moderation_request,
        media::patch_moderation_request_list,
        media::internal::internal_get_check_moderation_request_for_account,
        media::internal::internal_post_update_profile_image_visibility,
    ),
    components(schemas(
        common::EventToClient,
        account::data::AccountIdLight,
        account::data::ApiKey,
        account::data::Account,
        account::data::AccountState,
        account::data::AccountSetup,
        account::data::Capabilities,
        account::data::BooleanSetting,
        account::data::DeleteStatus,
        account::data::SignInWithLoginInfo,
        account::data::LoginResult,
        account::data::RefreshToken,
        account::data::AuthPair,
        profile::data::Profile,
        profile::data::ProfilePage,
        profile::data::ProfileLink,
        profile::data::ProfileVersion,
        profile::data::ProfileUpdate,
        profile::data::Location,
        media::data::ModerationRequest,
        media::data::ModerationRequestContent,
        media::data::ModerationRequestId,
        media::data::ModerationRequestState,
        media::data::ModerationList,
        media::data::Moderation,
        media::data::HandleModerationRequest,
        media::data::SlotId,
        media::data::ContentId,
        media::data::PrimaryImage,
        media::data::SecurityImage,
        media::data::ImageAccessCheck,
        media::data::NormalImages,
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

pub trait GetApiKeys {
    /// Users which are logged in.
    fn api_keys(&self) -> ApiKeyManager<'_>;
}

pub trait GetUsers {
    /// All users registered in the service.
    fn users(&self) -> AccountIdManager<'_>;
}

pub trait WriteDatabase {
    fn write_database(&self) -> &WriteCommandRunnerHandle;
}

pub trait ReadDatabase {
    fn read_database(&self) -> ReadCommands<'_>;
}

pub trait SignInWith {
    fn sign_in_with_manager(&self) -> &SignInWithManager;
}

pub trait GetInternalApi {
    fn internal_api(&self) -> InternalApiManager;
}

pub trait GetConfig {
    fn config(&self) -> &Config;
}
