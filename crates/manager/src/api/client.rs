use manager_api::{protocol::RequestSenderCmds, ClientConfig, ClientError, ManagerClient};
use manager_model::{JsonRpcRequest, JsonRpcResponse, ManagerInstanceName};

use crate::server::app::S;

use super::{server::json_rpc::handle_request_type, GetConfig};

use error_stack::{Result, ResultExt};

pub struct LocalOrRemoteApiClient<'a> {
    request_receiver: ManagerInstanceName,
    state: &'a S,
}

impl<'a> LocalOrRemoteApiClient<'a> {
    pub fn new(
        request_receiver: ManagerInstanceName,
        state: &'a S,
    ) -> Self {
        Self {
            request_receiver,
            state,
        }
    }

    async fn handle_api_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        if self.state.config().manager_name() == request.receiver {
            handle_request_type(
                request.request,
                self.state,
            )
                .await
                .change_context(ClientError::LocalApiRequest)
        } else if let Some(m) = self.state.config().find_remote_manager(&request.receiver)  {
            let config = ClientConfig {
                url: m.url.clone(),
                root_certificate: self.state.config().root_certificate(),
                api_key: self.state.config().api_key().to_string(),
            };
            let client = ManagerClient::connect(config)
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            let response = client.send_request(request)
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            Ok(response)
        } else {
            Ok(JsonRpcResponse::request_receiver_not_found())
        }
    }
}

impl RequestSenderCmds for LocalOrRemoteApiClient<'_> {
    fn request_receiver_name(&self) -> ManagerInstanceName {
        self.request_receiver.clone()
    }
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        self.handle_api_request(request).await
    }
}
