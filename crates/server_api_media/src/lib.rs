#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod internal_api;
pub mod media;
pub mod media_admin;
pub mod media_internal;

pub use server_api::{app, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
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
        media::delete_moderation_request,
        // Media admin
        media_admin::patch_moderation_request_list,
        media_admin::post_handle_moderation_request,
        // Media internal
        media_internal::internal_get_check_moderation_request_for_account,
    ),
    components(schemas(
        // Media
        model::media::ModerationRequest,
        model::media::ModerationRequestIdDb,
        model::media::ModerationRequestContent,
        model::media::ModerationRequestState,
        model::media::CurrentModerationRequest,
        model::media::MediaContentType,
        model::media::SlotId,
        model::media::ContentId,
        model::media::ContentProcessingId,
        model::media::ContentProcessingState,
        model::media::ContentProcessingStateType,
        model::media::NewContentParams,
        model::media::ProfileContent,
        model::media::ProfileContentVersion,
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
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocMedia;

pub use server_api::{db_write, db_write_multiple};
