
use base64::Engine;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct CurrentVersions {
    pub versions: String,
}
