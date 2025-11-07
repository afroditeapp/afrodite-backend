use model_server_data::ProfileLink;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ProfilePage {
    profiles: Vec<ProfileLink>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_invalid_iterator_session_id: bool,
}
impl ProfilePage {
    pub fn successful(profiles: Vec<ProfileLink>) -> Self {
        Self {
            profiles,
            ..Default::default()
        }
    }

    pub fn error_invalid_iterator_session_id() -> Self {
        Self {
            error: true,
            error_invalid_iterator_session_id: true,
            ..Default::default()
        }
    }
}
