//! HTTP API types and request handlers for all servers.

use config::Config;
use futures::Future;
use model::{AccountId, BackendVersion};
use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;
use crate::{
    app::sign_in_with::SignInWithManager,
    data::{
        read::ReadCommands,
        utils::{AccessTokenManager, AccountIdManager},
        write_commands::WriteCmds,
        write_concurrent::ConcurrentWriteHandle,
        DataError,
    },
    internal::InternalApiManager,
    manager_client::ManagerApiManager,
};

// Routes
pub mod account;
pub mod account_internal;
pub mod chat;
pub mod common;
pub mod common_admin;
pub mod media;
pub mod media_admin;
pub mod media_internal;
pub mod profile;
pub mod profile_internal;

pub mod utils;

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
        common_admin::post_request_restart_or_reset_backend,
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
        account_internal::check_access_token,
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
        model::common::AccountId,
        model::common::AccessToken,
        model::common::RefreshToken,
        // Account
        model::account::Account,
        model::account::AccountState,
        model::account::AccountSetup,
        model::account::Capabilities,
        model::account::BooleanSetting,
        model::account::DeleteStatus,
        model::account::SignInWithLoginInfo,
        model::account::LoginResult,
        model::account::AuthPair,
        // Profile
        model::profile::Profile,
        model::profile::ProfilePage,
        model::profile::ProfileLink,
        model::profile::ProfileVersion,
        model::profile::ProfileUpdate,
        model::profile::Location,
        model::media::ModerationRequest,
        model::media::ModerationRequestIdDb,
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
        // Manager
        manager_model::SystemInfoList,
        manager_model::SystemInfo,
        manager_model::CommandOutput,
        manager_model::BuildInfo,
        manager_model::SoftwareInfo,
        manager_model::RebootQueryParam,
        manager_model::ResetDataQueryParam,
        manager_model::DownloadType,
        manager_model::DownloadTypeQueryParam,
        manager_model::SoftwareOptions,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
    info(
        title = "pihka-backend",
        description = "Pihka backend API",
        version = "0.1.0"
    )
)]
pub struct ApiDoc;

// App state getters

pub trait GetAccessTokens {
    /// Users which are logged in.
    fn access_tokens(&self) -> AccessTokenManager<'_>;
}

pub trait GetAccounts {
    /// All accounts registered in the service.
    fn accounts(&self) -> AccountIdManager<'_>;
}

#[async_trait::async_trait]
pub trait WriteData {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = error_stack::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> error_stack::Result<CmdResult, DataError>;

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = error_stack::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> error_stack::Result<CmdResult, DataError>;
}

pub trait ReadData {
    fn read(&self) -> ReadCommands<'_>;
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
/// Converts crate::data::DataError to crate::api::utils::StatusCode.
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
/// }
/// ```
macro_rules! db_write {
    ($state:expr, move |$cmds:ident| $commands:expr) => {
        async {
            let r: error_stack::Result<_, crate::data::DataError> = $state
                .write(move |$cmds| async move { ($commands).await })
                .await;
            let r: std::result::Result<_, crate::api::utils::StatusCode> = r.map_err(|e| e.into());
            r
        }
        .await
    };
}

// Make db_write available in all modules
pub(crate) use db_write;
