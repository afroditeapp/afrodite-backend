use crate::{
    apis::{
        configuration,
        media_api::{GetImageError, PutImageToModerationSlotError},
        Error, ResponseContent,
    },
    models::{AccountIdLight, ContentId},
};

impl Copy for AccountIdLight {}

impl AccountIdLight {
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

// Fixed request functions

/// Get profile image
pub async fn get_image_fixed(
    configuration: &configuration::Configuration,
    account_id: &str,
    content_id: &str,
) -> Result<Vec<u8>, Error<GetImageError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/media_api/image/{account_id}/{content_id}",
        local_var_configuration.base_path,
        account_id = crate::apis::urlencode(account_id),
        content_id = crate::apis::urlencode(content_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

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
        local_var_req_builder = local_var_req_builder.header("x-api-key", local_var_value);
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

/// Set image to moderation request slot.  Slots from 0 to 2 are available.  TODO: resize and check images at some point
pub async fn put_image_to_moderation_slot_fixed(
    configuration: &configuration::Configuration,
    slot_id: i32,
    body: Vec<u8>,
) -> Result<crate::models::ContentId, Error<PutImageToModerationSlotError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/media_api/moderation/request/slot/{slot_id}",
        local_var_configuration.base_path,
        slot_id = slot_id
    );
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
        local_var_req_builder = local_var_req_builder.header("x-api-key", local_var_value);
    };
    local_var_req_builder = local_var_req_builder.body(body);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PutImageToModerationSlotError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
