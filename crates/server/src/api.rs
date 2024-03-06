//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod account;
pub mod account_internal;
pub mod chat;
pub mod common;
pub mod common_admin;
pub mod common_internal;
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
        // Common
        common::get_version,
        common::get_connect_websocket,
        // Common admin
        common_admin::get_system_info,
        common_admin::get_software_info,
        common_admin::get_latest_build_info,
        common_admin::get_backend_config,
        common_admin::get_perf_data,
        common_admin::post_request_build_software,
        common_admin::post_request_update_software,
        common_admin::post_request_restart_or_reset_backend,
        common_admin::post_backend_config,
        // Account
        account::post_register,
        account::post_login,
        account::post_sign_in_with_login,
        account::post_account_setup,
        account::post_account_data,
        account::post_complete_setup,
        account::post_delete,
        account::put_setting_profile_visiblity,
        account::get_account_state,
        account::get_account_setup,
        account::get_account_data,
        account::get_deletion_status,
        account::delete_cancel_deletion,
        // Profile
        profile::get_profile,
        profile::get_profile_from_database_debug_mode_benchmark,
        profile::get_location,
        profile::get_favorite_profiles,
        profile::get_available_profile_attributes,
        profile::get_search_groups,
        profile::post_get_next_profile_page,
        profile::post_profile,
        profile::post_profile_to_database_debug_mode_benchmark,
        profile::post_reset_profile_paging,
        profile::post_favorite_profile,
        profile::post_search_groups,
        profile::put_location,
        profile::delete_favorite_profile,
        // Media
        media::get_profile_content_info,
        media::get_pending_profile_content_info,
        media::get_all_account_media_content,
        media::get_content,
        media::get_security_content_info,
        media::put_security_content_info,
        media::get_pending_security_content_info,
        media::put_pending_security_content_info,
        media::delete_pending_security_content_info,
        media::get_moderation_request,
        media::get_map_tile,
        media::put_moderation_request,
        media::put_content_to_content_slot,
        media::get_content_slot_state,
        media::put_profile_content,
        media::put_pending_profile_content,
        media::delete_content,
        // Media admin
        media_admin::patch_moderation_request_list,
        media_admin::post_handle_moderation_request,
        // Media internal
        media_internal::internal_get_check_moderation_request_for_account,
        // Chat
        chat::get_sent_likes,
        chat::get_received_likes,
        chat::get_matches,
        chat::get_sent_blocks,
        chat::get_received_blocks,
        chat::get_pending_messages,
        chat::get_message_number_of_latest_viewed_message,
        chat::post_send_like,
        chat::post_send_message,
        chat::post_block_profile,
        chat::post_unblock_profile,
        chat::delete_like,
        chat::delete_pending_messages,
        chat::post_message_number_of_latest_viewed_message,
    ),
    components(schemas(
        // Common
        model::common::EventToClient,
        model::common::EventType,
        model::common::BackendVersion,
        model::common::AccountId,
        model::common::AccessToken,
        model::common::RefreshToken,
        model::common::LatestViewedMessageChanged,
        model::common::ContentProcessingStateChanged,
        model::common::sync_version::SyncVersion,
        model::common::sync_version::AccountSyncVersion,
        simple_backend_model::UnixTime,
        // Common admin
        model::common_admin::BackendConfig,
        model::common_admin::BotConfig,
        simple_backend_model::perf::TimeGranularity,
        simple_backend_model::perf::PerfHistoryQuery,
        simple_backend_model::perf::PerfValueArea,
        simple_backend_model::perf::PerfHistoryValue,
        simple_backend_model::perf::PerfHistoryQueryResult,
        // Account
        model::account::Account,
        model::account::AccountState,
        model::account::AccountSetup,
        model::account::AccountData,
        model::account::Capabilities,
        model::account::BooleanSetting,
        model::account::DeleteStatus,
        model::account::SignInWithLoginInfo,
        model::account::LoginResult,
        model::account::AuthPair,
        model::account::ProfileVisibility,
        // Profile
        model::profile::Profile,
        model::profile::ProfilePage,
        model::profile::ProfileLink,
        model::profile::ProfileVersion,
        model::profile::ProfileUpdate,
        model::profile::ProfileAge,
        model::profile::Location,
        model::profile::FavoriteProfilesPage,
        model::profile::AvailableProfileAttributes,
        model::profile::ProfileAttributes,
        model::profile::ProfileAttributesSyncVersion,
        model::profile::ProfileAttributeValue,
        model::profile::ProfileAttributeValueUpdate,
        model::profile::Attribute,
        model::profile::AttributeOrderMode,
        model::profile::AttributeMode,
        model::profile::AttributeValue,
        model::profile::AttributeValueOrderMode,
        model::profile::Language,
        model::profile::Translation,
        model::profile::GroupValues,
        model::profile::IconResource,
        model::profile::IconLocation,
        model::profile::SearchGroups,
        // Media
        model::media::ModerationRequest,
        model::media::ModerationRequestIdDb,
        model::media::ModerationRequestContent,
        model::media::ModerationRequestState,
        model::media::MediaContentType,
        model::media::SlotId,
        model::media::ContentId,
        model::media::ContentProcessingId,
        model::media::ContentProcessingState,
        model::media::ContentProcessingStateType,
        model::media::NewContentParams,
        model::media::ProfileContent,
        model::media::SetProfileContent,
        model::media::PendingProfileContent,
        model::media::SecurityContent,
        model::media::PendingSecurityContent,
        model::media::ContentAccessCheck,
        model::media::ContentInfo,
        model::media::ContentInfoDetailed,
        model::media::ContentState,
        model::media::ContentInfo,
        model::media::ContentSlot,
        model::media::AccountContent,
        model::media::MapTileZ,
        model::media::MapTileX,
        model::media::MapTileY,
        // Media admin
        model::media_admin::ModerationRequestId,
        model::media_admin::ModerationList,
        model::media_admin::Moderation,
        model::media_admin::ModerationQueueType,
        model::media_admin::ModerationQueueTypeParam,
        model::media_admin::HandleModerationRequest,
        // Chat
        model::chat::SentLikesPage,
        model::chat::ReceivedLikesPage,
        model::chat::MatchesPage,
        model::chat::SentBlocksPage,
        model::chat::ReceivedBlocksPage,
        model::chat::PendingMessagesPage,
        model::chat::PendingMessage,
        model::chat::PendingMessageId,
        model::chat::PendingMessageDeleteList,
        model::chat::MessageNumber,
        model::chat::SendMessageToAccount,
        model::chat::UpdateMessageViewStatus,
        model::chat::ReceivedBlocksSyncVersion,
        model::chat::ReceivedLikesSyncVersion,
        model::chat::SentBlocksSyncVersion,
        model::chat::SentLikesSyncVersion,
        model::chat::MatchesSyncVersion,
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

impl ApiDoc {
    pub fn open_api_json_string() -> Result<String, serde_json::Error> {
        Self::openapi().to_pretty_json()
    }
}

/// Macro for writing data with different code style.
/// Makes "async move" and "await" keywords unnecessary.
/// The macro "closure" should work like a real closure.
///
/// This macro will guarantee that contents of the closure will run
/// completely even if HTTP connection fails when closure is running.
///
/// Converts crate::data::DataError to crate::api::utils::StatusCode.
///
/// Example usage:
///
/// ```
/// use server::api::db_write;
/// use server::api::utils::StatusCode;
/// use server::app::WriteData;
/// pub async fn axum_route_handler<S: WriteData>(
///     state: S,
/// ) -> std::result::Result<(), StatusCode> {
///     db_write!(state, move |cmds|
///         async move { Ok(()) }
///     )
/// }
/// ```
#[macro_export]
macro_rules! db_write {
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        let r = async {
            let r: $crate::result::Result<_, $crate::data::DataError> = $state
                .write(move |$cmds| async move { ($commands).await })
                .await;
            r
        }
        .await;

        use $crate::api::utils::ConvertDataErrorToStatusCode;
        r.convert_data_error_to_status_code()
    }};
}

// Make db_write available in all modules
pub use db_write;

/// Same as db_write! but allows multiple commands to be executed because the
/// commands are not automatically awaited.
macro_rules! db_write_multiple {
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        let r = async {
            let r: $crate::result::Result<_, $crate::data::DataError> = $state
                .write(move |$cmds| async move { ($commands) })
                .await;
            r
        }
        .await;

        use $crate::api::utils::ConvertDataErrorToStatusCode;
        r.convert_data_error_to_status_code()
    }};
}

// Make db_write available in all modules
pub(crate) use db_write_multiple;
