use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct GetVideoCallUrlsResult {
    /// Standard Jitsi Meet URL to a meeting with HTTPS
    /// schema. Can be used to crate URL to open Jitsi Meet app.
    pub url: String,
    /// Custom Jitsi Meet URL to a meeting with HTTPS
    /// schema. If exists, this should be used to open the meeting
    /// when Jitsi Meet app is not installed.
    pub custom_url: Option<String>,
}
