use base64::Engine;
use error_stack::{Result, ResultExt};
use jsonwebtoken::{EncodingKey, Header};
use serde::Serialize;
use sha2::{Digest, Sha256};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::UnixTime;

pub struct VideoCallUserInfo {
    pub id: String,
    pub name: String,
}

impl VideoCallUserInfo {
    /// This creates the same result even if self and other is swapped.
    fn create_room(&self, other: &VideoCallUserInfo) -> String {
        let mut ids = [&self.id, &other.id];
        ids.sort();
        let mut hasher = Sha256::new();
        hasher.update(ids[0]);
        hasher.update(ids[1]);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
    }
}

pub struct MeetingUrls {
    pub url: String,
    pub custom_url: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum JitsiMeetUrlCreatorError {
    #[error("Token encoding failed")]
    TokenEncoding,
}

pub struct JitsiMeetUrlCreator<'a> {
    config: &'a SimpleBackendConfig,
}

impl<'a> JitsiMeetUrlCreator<'a> {
    pub fn new(config: &'a SimpleBackendConfig) -> Self {
        Self { config }
    }

    /// None is returned when video calls are not configured
    pub fn create_url(
        &self,
        url_requester: VideoCallUserInfo,
        callee: VideoCallUserInfo,
    ) -> Result<Option<MeetingUrls>, JitsiMeetUrlCreatorError> {
        let Some(config) = &self.config.video_calling().jitsi_meet else {
            return Ok(None);
        };

        let room = format!(
            "{}{}",
            config.room_prefix,
            url_requester.create_room(&callee),
        );

        let exp = UnixTime::current_time().add_seconds(config.jwt_validity_time.seconds);

        let claims = Claims {
            aud: config.jwt_aud.clone(),
            iss: config.jwt_iss.clone(),
            exp: exp.ut,
            room: room.clone(),
            context: Context {
                user: url_requester.into(),
                callee: callee.into(),
            },
        };

        let jwt = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .change_context(JitsiMeetUrlCreatorError::TokenEncoding)?;

        let mut url = config.url.clone();
        url.set_path(&room);
        let query = format!("jwt={}", &jwt);
        url.set_query(Some(&query));
        Ok(Some(MeetingUrls {
            url: url.to_string(),
            custom_url: config
                .custom_url
                .as_ref()
                .map(|v| v.replace("{room}", &room).replace("{jwt}", &jwt)),
        }))
    }
}

#[derive(Serialize)]
struct Name {
    name: String,
}

impl From<VideoCallUserInfo> for Name {
    fn from(value: VideoCallUserInfo) -> Self {
        Self { name: value.name }
    }
}

#[derive(Serialize)]
struct Context {
    user: Name,
    callee: Name,
}

#[derive(Serialize)]
struct Claims {
    aud: String,
    iss: String,
    exp: i64,
    room: String,
    context: Context,
}
