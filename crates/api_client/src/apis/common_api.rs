/*
 * afrodite-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;
use serde::{Deserialize, Serialize};
use crate::{apis::ResponseContent, models};
use super::{Error, configuration};


/// struct for typed errors of method [`get_connect_websocket`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetConnectWebsocketError {
    Status401(),
    Status500(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_version`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetVersionError {
    UnknownValue(serde_json::Value),
}


/// Protocol: 1. Client sends version information as Binary message, where    - u8: Client WebSocket protocol version (currently 0).    - u8: Client type number. (0 = Android, 1 = iOS, 2 = Web, 255 = Test mode bot)    - u16: Client Major version.    - u16: Client Minor version.    - u16: Client Patch version.     The u16 values are in little endian byte order. 2. Client sends current refresh token as Binary message. 3. If server supports the client, the server sends next refresh token    as Binary message.    If server does not support the client, the server sends Text message    and closes the connection without WebSocket Close message. 4. Server sends new access token as Binary message. The client must    convert the token to base64url encoding without padding.    (At this point API can be used.) 5. Client sends list of current data sync versions as Binary message, where    items are [u8; 2] and the first u8 of an item is the data type number    and the second u8 of an item is the sync version number for that data.    If client does not have any version of the data, the client should    send 255 as the version number.     Available data types:    - 0: Account 6. Server starts to send JSON events as Text messages and empty binary    messages to test connection to the client. Client can ignore the empty    binary messages. 7. If needed, the client sends empty binary messages to test connection to    the server.  The new access token is valid until this WebSocket is closed or the server detects a timeout. To prevent the timeout the client must send a WebScoket ping message before 6 minutes elapses from connection establishment or previous ping message.  `Sec-WebSocket-Protocol` header must have 2 protocols/values. The first is \"0\" and that protocol is accepted. The second is access token of currently logged in account. The token is base64url encoded without padding.
pub async fn get_connect_websocket(configuration: &configuration::Configuration, ) -> Result<(), Error<GetConnectWebsocketError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/common_api/connect", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<GetConnectWebsocketError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn get_version(configuration: &configuration::Configuration, ) -> Result<models::BackendVersion, Error<GetVersionError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/common_api/version", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetVersionError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

