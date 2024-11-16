/*
 * dating-app-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ProfileNameModerationState {
    #[serde(rename = "Empty")]
    Empty,
    #[serde(rename = "WaitingBotOrHumanModeration")]
    WaitingBotOrHumanModeration,
    #[serde(rename = "WaitingHumanModeration")]
    WaitingHumanModeration,
    #[serde(rename = "AcceptedByBot")]
    AcceptedByBot,
    #[serde(rename = "AcceptedByHuman")]
    AcceptedByHuman,
    #[serde(rename = "AcceptedUsingAllowlist")]
    AcceptedUsingAllowlist,
    #[serde(rename = "RejectedByBot")]
    RejectedByBot,
    #[serde(rename = "RejectedByHuman")]
    RejectedByHuman,

}

impl std::fmt::Display for ProfileNameModerationState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::WaitingBotOrHumanModeration => write!(f, "WaitingBotOrHumanModeration"),
            Self::WaitingHumanModeration => write!(f, "WaitingHumanModeration"),
            Self::AcceptedByBot => write!(f, "AcceptedByBot"),
            Self::AcceptedByHuman => write!(f, "AcceptedByHuman"),
            Self::AcceptedUsingAllowlist => write!(f, "AcceptedUsingAllowlist"),
            Self::RejectedByBot => write!(f, "RejectedByBot"),
            Self::RejectedByHuman => write!(f, "RejectedByHuman"),
        }
    }
}

impl Default for ProfileNameModerationState {
    fn default() -> ProfileNameModerationState {
        Self::Empty
    }
}

