use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod news;

pub use news::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct CurrentVersions {
    pub versions: String,
}
