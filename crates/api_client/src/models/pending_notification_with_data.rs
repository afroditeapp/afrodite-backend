/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// PendingNotificationWithData : Pending notification with notification data.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingNotificationWithData {
    /// Data for NEW_MESSAGE notification.  List of account IDs which have sent a new message.
    #[serde(rename = "new_message_received_from", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub new_message_received_from: Option<Option<Vec<models::AccountId>>>,
    /// Data for RECEIVED_LIKES_CHANGED notification.
    #[serde(rename = "received_likes_changed", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub received_likes_changed: Option<Option<Box<models::NewReceivedLikesCountResult>>>,
    /// Pending notification (or multiple notifications which each have different type) not yet received notifications which push notification requests client to download.  The integer is a bitflag.  - const NEW_MESSAGE = 0x1; - const RECEIVED_LIKES_CHANGED = 0x2; 
    #[serde(rename = "value")]
    pub value: i64,
}

impl PendingNotificationWithData {
    /// Pending notification with notification data.
    pub fn new(value: i64) -> PendingNotificationWithData {
        PendingNotificationWithData {
            new_message_received_from: None,
            received_likes_changed: None,
            value,
        }
    }
}

