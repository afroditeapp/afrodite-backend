use error_stack::{Result, ResultExt};
use manager_api::{ClientError, protocol::RequestSenderCmds};
use manager_model::{JsonRpcRequest, JsonRpcResponse, ManagerInstanceName};

use super::server::json_rpc::handle_rpc_request;
use crate::server::app::S;

pub struct LocalOrRemoteApiClient<'a> {
    request_recipient: ManagerInstanceName,
    state: &'a S,
}

impl<'a> LocalOrRemoteApiClient<'a> {
    pub fn new(request_recipient: ManagerInstanceName, state: &'a S) -> Self {
        Self {
            request_recipient,
            state,
        }
    }
}

impl RequestSenderCmds for LocalOrRemoteApiClient<'_> {
    fn request_recipient_name(&self) -> ManagerInstanceName {
        self.request_recipient.clone()
    }
    async fn send_request(self, request: JsonRpcRequest) -> Result<JsonRpcResponse, ClientError> {
        handle_rpc_request(request, None, self.state)
            .await
            .change_context(ClientError::JsonRpc)
    }
}
