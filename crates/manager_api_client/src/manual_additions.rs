use crate::{apis::{configuration, Error, manager_api::GetLatestSoftwareError, ResponseContent}, models::{SoftwareOptions, DownloadType}};




/// Download latest software
///
/// Returns bytes or SoftwareInfo JSON depending on the download_type parameter.
pub async fn get_latest_software_fixed(configuration: &configuration::Configuration, software_options: SoftwareOptions, download_type: DownloadType) -> Result<Vec<u8>, Error<GetLatestSoftwareError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/manager_api/latest_software", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("software_options", &software_options.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("download_type", &download_type.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
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
    let local_var_content = local_var_resp.bytes().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(local_var_content.to_vec())
    } else {
        let status_string = local_var_status.to_string();
        let local_var_entity: Option<GetLatestSoftwareError> = serde_json::from_str(&status_string).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: status_string, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}
