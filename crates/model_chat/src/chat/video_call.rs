use serde::Serialize;
use utoipa::ToSchema;

#[derive(Default, Serialize, ToSchema)]
pub struct PostVideoCallUrlResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jitsi_meet: Option<JitsiMeetUrl>,
}

#[derive(Serialize, ToSchema)]
pub struct JitsiMeetUrl {
    /// Standard Jitsi Meet URL to a meeting with HTTPS
    /// schema. Can be used to create an URL which opens Jitsi Meet app.
    pub url: String,
    /// Custom Jitsi Meet URL to a meeting with HTTPS
    /// schema. If exists, this should be used to open the meeting
    /// when Jitsi Meet app is not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_url: Option<String>,
}
