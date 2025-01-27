/*
 * afrodite-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingMessageId {
    #[serde(rename = "mn")]
    pub mn: Box<models::MessageNumber>,
    /// Sender of the message.
    #[serde(rename = "sender")]
    pub sender: Box<models::AccountId>,
}

impl PendingMessageId {
    pub fn new(mn: models::MessageNumber, sender: models::AccountId) -> PendingMessageId {
        PendingMessageId {
            mn: Box::new(mn),
            sender: Box::new(sender),
        }
    }
}

