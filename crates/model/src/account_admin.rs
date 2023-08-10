

use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct CurrentVersions {
    pub versions: String,
}
