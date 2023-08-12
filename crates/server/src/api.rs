//! HTTP API types and request handlers for all servers.

use futures::Future;
use utoipa::OpenApi;

use config::Config;
use model::{AccountIdLight, BackendVersion};

// use crate::{
//     server::{
//         app::sign_in_with::SignInWithManager,
//         data::{
//             read::ReadCommands,
//             utils::{AccountIdManager, ApiKeyManager},
//             write_commands::WriteCmds,
//             write_concurrent::ConcurrentWriteHandle,
//             DatabaseError,
//         },
//         internal::InternalApiManager,
//         manager_client::ManagerApiManager,
//     },
// };
use crate::{
    app::sign_in_with::SignInWithManager,
    data::{
        DatabaseError,
        read::ReadCommands,
        utils::{AccountIdManager, ApiKeyManager},
        write_commands::WriteCmds,
        write_concurrent::ConcurrentWriteHandle,
    },
    internal::InternalApiManager,
    manager_client::ManagerApiManager,
};

use self::utils::SecurityApiTokenDefault;

// Routes
pub mod account;
pub mod account_internal;
pub mod common;
pub mod common_admin;
pub mod media;
pub mod media_admin;
pub mod media_internal;
pub mod profile;
pub mod profile_internal;

pub mod utils;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        common::get_version,
        common::get_connect_websocket,
        common_admin::get_system_info,
        common_admin::get_software_info,
        common_admin::get_latest_build_info,
        common_admin::post_request_build_software,
        common_admin::post_request_update_software,
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
        account_internal::check_api_key,
        account_internal::internal_get_account_state,
        profile::get_profile,
        profile::get_profile_from_database_debug_mode_benchmark,
        profile::post_get_next_profile_page,
        profile::post_profile,
        profile::post_profile_to_database_debug_mode_benchmark,
        profile::post_reset_profile_paging,
        profile::put_location,
        profile_internal::internal_post_update_profile_visibility,
        media::get_primary_image_info,
        media::get_all_normal_images,
        media::get_image,
        media::get_moderation_request,
        media::put_moderation_request,
        media::put_image_to_moderation_slot,
        media::put_primary_image,
        media_admin::patch_moderation_request_list,
        media_admin::post_handle_moderation_request,
        media_admin::get_security_image_info,
        media_internal::internal_get_check_moderation_request_for_account,
        media_internal::internal_post_update_profile_image_visibility,
    ),
    components(schemas(
        // Common
        model::common::EventToClient,
        model::common::BackendVersion,
        // Account
        model::account::AccountIdLight,
        model::account::ApiKey,
        model::account::Account,
        model::account::AccountState,
        model::account::AccountSetup,
        model::account::Capabilities,
        model::account::BooleanSetting,
        model::account::DeleteStatus,
        model::account::SignInWithLoginInfo,
        model::account::LoginResult,
        model::account::RefreshToken,
        model::account::AuthPair,
        // Profile
        model::profile::Profile,
        model::profile::ProfilePage,
        model::profile::ProfileLink,
        model::profile::ProfileVersion,
        model::profile::ProfileUpdate,
        model::profile::Location,
        model::media::ModerationRequest,
        model::media::ModerationRequestContent,
        model::media::ModerationRequestState,
        model::media_admin::ModerationRequestId,
        model::media_admin::ModerationList,
        model::media_admin::Moderation,
        model::media_admin::HandleModerationRequest,
        model::media::SlotId,
        model::media::ContentId,
        model::media::PrimaryImage,
        model::media::SecurityImage,
        model::media::ImageAccessCheck,
        model::media::NormalImages,
        manager_model::SystemInfoList,
        manager_model::SystemInfo,
        manager_model::CommandOutput,
        manager_model::BuildInfo,
        manager_model::SoftwareInfo,
        manager_model::RebootQueryParam,
        manager_model::DownloadType,
        manager_model::DownloadTypeQueryParam,
        manager_model::SoftwareOptions,
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

#[async_trait::async_trait]
pub trait WriteData {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = error_stack::Result<CmdResult, DatabaseError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> error_stack::Result<CmdResult, DatabaseError>;

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = error_stack::Result<CmdResult, DatabaseError>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountIdLight,
        cmd: GetCmd,
    ) -> error_stack::Result<CmdResult, DatabaseError>;
}

pub trait ReadDatabase {
    fn read_database(&self) -> ReadCommands<'_>;
}

pub trait SignInWith {
    fn sign_in_with_manager(&self) -> &SignInWithManager;
}

pub trait GetInternalApi {
    fn internal_api(&self) -> InternalApiManager<Self>
    where
        Self: Sized;
}

pub trait GetManagerApi {
    fn manager_api(&self) -> ManagerApiManager;
}

pub trait GetConfig {
    fn config(&self) -> &Config;
}

pub trait BackendVersionProvider {
    fn backend_version(&self) -> BackendVersion;
}

/// Macro for writing data with different code style.
/// Makes "async move" and "await" keywords unnecessary.
/// The macro "closure" should work like a real closure.
///
/// Example usage:
///
/// ```rust
/// pub async fn axum_route_handler<S: WriteDatabase>(
///     state: S,
/// ) -> Result<(), StatusCode> {
///     let api_caller_account_id = todo!();
///     let new_image = todo!();
///     db_write!(state, move |cmds|
///         cmds.media()
///             .update_primary_image(api_caller_account_id, new_image)
///     )
///     .await
///     .map_err(|e| {
///         error!("{}", e);
///         StatusCode::INTERNAL_SERVER_ERROR
///     })
///     Ok(())
/// }
/// ```
macro_rules! db_write {
    ($state:expr, move |$cmds:ident| $commands:expr) => {
        $state.write(move |$cmds| async move { ($commands).await })
    };
}

// Make db_write available in all modules
pub(crate) use db_write;
