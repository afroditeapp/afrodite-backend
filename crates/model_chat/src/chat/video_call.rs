use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct GetVideoCallUrlResult {
    pub url: String,
}
