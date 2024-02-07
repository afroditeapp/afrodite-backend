use crate::{
    apis::{
        configuration, media_api::{GetContentError, PutContentToContentSlotError}, Error, ResponseContent
    },
    models::{AccountId, ContentId, Location, UnixTime, MediaContentType},
};

impl Copy for AccountId {}

impl AccountId {
    pub fn to_string(&self) -> String {
        self.account_id.hyphenated().to_string()
    }
}

impl Copy for ContentId {}

impl ContentId {
    pub fn to_string(&self) -> String {
        self.content_id.hyphenated().to_string()
    }
}

impl UnixTime {
    pub fn to_string(&self) -> String {
        self.unix_time.to_string()
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
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("x-access-token", local_var_value);
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
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("x-access-token", local_var_value);
    };
    local_var_req_builder = local_var_req_builder.json(&body);

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

pub async fn api_available(configuration: &configuration::Configuration) -> Result<(), ()> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/", local_var_configuration.base_path,);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("x-access-token", local_var_value);
    };

    let local_var_req = local_var_req_builder.build().map_err(|_| ())?;
    let _ = local_var_client
        .execute(local_var_req)
        .await
        .map_err(|_| ())?;

    Ok(())
}
