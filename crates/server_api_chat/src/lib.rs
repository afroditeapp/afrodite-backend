#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod chat;

pub use server_api::{app, internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Chat
        chat::get_sent_likes,
        chat::get_received_likes,
        chat::get_matches,
        chat::get_sent_blocks,
        chat::get_received_blocks,
        chat::get_pending_messages,
        chat::get_message_number_of_latest_viewed_message,
        chat::get_public_key,
        chat::post_send_like,
        chat::post_send_message,
        chat::post_block_profile,
        chat::post_unblock_profile,
        chat::delete_like,
        chat::delete_pending_messages,
        chat::post_message_number_of_latest_viewed_message,
        chat::post_set_device_token,
        chat::post_get_pending_notification,
        chat::post_public_key,
    ),
    components(schemas(
        // Chat
        model::chat::SentLikesPage,
        model::chat::ReceivedLikesPage,
        model::chat::MatchesPage,
        model::chat::SentBlocksPage,
        model::chat::ReceivedBlocksPage,
        model::chat::PendingMessage,
        model::chat::PendingMessageId,
        model::chat::PendingMessageDeleteList,
        model::chat::MessageNumber,
        model::chat::SendMessageResult,
        model::chat::UpdateMessageViewStatus,
        model::chat::ReceivedBlocksSyncVersion,
        model::chat::ReceivedLikesSyncVersion,
        model::chat::SentBlocksSyncVersion,
        model::chat::SentLikesSyncVersion,
        model::chat::MatchesSyncVersion,
        model::chat::FcmDeviceToken,
        model::chat::PendingNotification,
        model::chat::PendingNotificationWithData,
        model::chat::PendingNotificationToken,
        model::chat::LimitedActionResult,
        model::chat::LimitedActionStatus,
        model::chat::PublicKey,
        model::chat::PublicKeyId,
        model::chat::PublicKeyVersion,
        model::chat::PublicKeyData,
        model::chat::GetPublicKey,
        model::chat::SetPublicKey,
        model::chat::EncryptedMessage,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocChat;

pub use server_api::{db_write, db_write_multiple};
