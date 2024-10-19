use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AccountId, NewsId, NewsItem, NewsTranslationVersion};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsTranslations {
    pub id: NewsId,
    pub public: bool,
    pub aid_creator: Option<AccountId>,
    pub translations: Vec<NewsItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct UpdateNewsTranslation {
    pub title: String,
    pub body: String,
    pub current_version: NewsTranslationVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct UpdateNewsTranslationResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_already_changed: bool,
}

impl UpdateNewsTranslationResult {
    pub fn success() -> Self {
        Self {
            error_already_changed: false,
        }
    }

    pub fn error_already_changed() -> Self {
        Self {
            error_already_changed: true,
        }
    }
}
