use std::fmt;

use crate::{
    apis::{
        chat_api::{GetPendingMessagesError, GetPublicKeyError, PostAddPublicKeyError, PostSendMessageError}, configuration, media_api::{GetContentError, PutContentToContentSlotError}, Error, ResponseContent
    },
    models::{AccountId, ContentId, Location, MediaContentType, UnixTime},
};

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.aid)
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cid)
    }
}

impl fmt::Display for UnixTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ut)
    }
}

impl Copy for Location {}

// Fixed request functions

/// Get content data
pub async fn get_content_fixed(configuration: &configuration::Configuration, account_id: &str, content_id: &str, is_match: bool) -> Result<Vec<u8>, Error<GetContentError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/media_api/content/{account_id}/{content_id}", local_var_configuration.base_path, account_id=crate::apis::urlencode(account_id), content_id=crate::apis::urlencode(content_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("is_match", &is_match.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.bytes().await?.into_iter().collect();

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(local_var_content)
    } else {
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: "".to_string(),
            entity: None,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Set content to content processing slot. Processing ID will be returned and processing of the content will begin. Events about the content processing will be sent to the client.  The state of the processing can be also queired. The querying is required to receive the content ID.  Slots from 0 to 6 are available.  One account can only have one content in upload or processing state. New upload might potentially delete the previous if processing of it is not complete.
pub async fn put_content_to_content_slot_fixed(
    configuration: &configuration::Configuration,
    slot_id: i32,
    secure_capture: bool,
    content_type: MediaContentType,
    body: Vec<u8>,
) -> Result<crate::models::ContentProcessingId, Error<PutContentToContentSlotError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/media_api/content_slot/{slot_id}", local_var_configuration.base_path, slot_id=slot_id);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("secure_capture", &secure_capture.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("content_type", &content_type.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.body(body);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PutContentToContentSlotError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Returns next public key ID number.  # Limits  Server can store limited amount of public keys. The limit is configurable from server config file and also user specific config exists. Max value between the two previous values is used to check is adding the key allowed.  Max key size is 8192 bytes.  The key must be OpenPGP public key with one signed user which ID is [model::AccountId] string.
pub async fn post_add_public_key_fixed(configuration: &configuration::Configuration, body: Vec<u8>) -> Result<crate::models::AddPublicKeyResult, Error<PostAddPublicKeyError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/chat_api/add_public_key", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.body(body);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PostAddPublicKeyError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn get_public_key_fixed(configuration: &configuration::Configuration, aid: &str, id: i64) -> Result<Vec<u8>, Error<GetPublicKeyError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/chat_api/public_key/{aid}", local_var_configuration.base_path, aid=crate::apis::urlencode(aid));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("id", &id.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.bytes().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(local_var_content.to_vec())
    } else {
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: "".to_string(),
            entity: None,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Max pending message count is 50. Max message size is u16::MAX.  The sender message ID must be value which server expects.  Sending will fail if one or two way block exists.
pub async fn post_send_message_fixed(configuration: &configuration::Configuration, sender_public_key_id: i64, receiver: &str, receiver_public_key_id: i64, client_id: i64, client_local_id: i64, body: Vec<u8>) -> Result<crate::models::SendMessageResult, Error<PostSendMessageError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/chat_api/send_message", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("sender_public_key_id", &sender_public_key_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("receiver", &receiver.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("receiver_public_key_id", &receiver_public_key_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("client_id", &client_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("client_local_id", &client_local_id.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.body(body);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PostSendMessageError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get list of pending messages.  The returned bytes is list of objects with following data: - UTF-8 text length encoded as 16 bit little endian number. - UTF-8 text which is PendingMessage JSON. - Binary message data length as 16 bit little endian number. - Binary message data
pub async fn get_pending_messages_fixed(configuration: &configuration::Configuration, ) -> Result<Vec<u8>, Error<GetPendingMessagesError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/chat_api/pending_messages", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.bytes().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(local_var_content.to_vec())
    } else {
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: "".to_string(),
            entity: None,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
