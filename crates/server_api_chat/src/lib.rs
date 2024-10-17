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
        chat::post_add_receiver_acknowledgement,
        chat::post_message_number_of_latest_viewed_message,
        chat::post_set_device_token,
        chat::post_get_pending_notification,
        chat::post_public_key,
        chat::get_sent_message_ids,
        chat::post_add_sender_acknowledgement,
        chat::post_reset_received_likes_paging,
        chat::post_get_next_received_likes_page,
        chat::post_get_new_received_likes_count,
        chat::post_reset_matches_paging,
        chat::post_get_next_matches_page,
    ),
    components(schemas(
        // Chat
        model::chat::PendingMessage,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocChat;

pub use server_api::{db_write, db_write_multiple};
