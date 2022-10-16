
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ImageFileName {
    image_file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ImageFile {
    #[schema(value_type = String, format = Binary)]
    data: Vec<u8>,
}
