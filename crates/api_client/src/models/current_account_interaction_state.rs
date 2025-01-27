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

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum CurrentAccountInteractionState {
    #[serde(rename = "Empty")]
    Empty,
    #[serde(rename = "LikeSent")]
    LikeSent,
    #[serde(rename = "LikeReceived")]
    LikeReceived,
    #[serde(rename = "Match")]
    Match,
    #[serde(rename = "BlockSent")]
    BlockSent,

}

impl std::fmt::Display for CurrentAccountInteractionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::LikeSent => write!(f, "LikeSent"),
            Self::LikeReceived => write!(f, "LikeReceived"),
            Self::Match => write!(f, "Match"),
            Self::BlockSent => write!(f, "BlockSent"),
        }
    }
}

impl Default for CurrentAccountInteractionState {
    fn default() -> CurrentAccountInteractionState {
        Self::Empty
    }
}

