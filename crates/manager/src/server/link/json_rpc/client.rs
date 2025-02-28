
use std::time::Duration;

use error_stack::{FutureExt, Result, ResultExt};
use manager_api::{protocol::{ClientConnectionRead, ClientConnectionWrite, ConnectionUtilsRead, ConnectionUtilsWrite}, ClientConfig, ManagerClient};
use manager_config::file::JsonRpcLinkConfigClient;
use manager_model::{JsonRpcLinkHeader, JsonRpcLinkMessage, JsonRpcLinkMessageType, JsonRpcRequest};
use simple_backend_utils::ContextExt;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{error, info, warn};

use crate::{api::server::json_rpc::handle_rpc_request, server::{app::S, ServerQuitWatcher}};

use crate::api::GetConfig;


#[derive(thiserror::Error, Debug)]
enum JsonRcpLinkClientError {
    #[error("Reading error")]
    Read,

    #[error("Writing error")]
    Write,

    #[error("Broken message channel")]
    BrokenMessageChannel,

    #[error("Link connection client error")]
    Client,

    #[error("Serialize")]
    Serialize,

    #[error("Deserialize")]
    Deserialize,
}

#[derive(Debug)]
pub struct JsonRcpLinkManagerClientQuitHandle {
    task: JoinHandle<()>,
}

impl JsonRcpLinkManagerClientQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("JsonRcpLink manager client quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct JsonRcpLinkManagerClient {
    state: S,
}

impl JsonRcpLinkManagerClient {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> JsonRcpLinkManagerClientQuitHandle {
        let manager = Self {
            state: state.clone(),
        };

        let task = tokio::spawn(manager.run(quit_notification.resubscribe()));

        JsonRcpLinkManagerClientQuitHandle {
            task,
        }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        if let Some(config) = self.state.config().json_rpc_link().client.clone() {
            tokio::select! {
                _ = self.create_connection_loop(config) => (),
                _ = quit_notification.recv() => (),
            }
        } else {
            let _ = quit_notification.recv().await;
        }
    }

    async fn create_connection_loop(
        mut self,
        config: JsonRpcLinkConfigClient,
    ) {
        let mut retry_wait_seconds = 2;
        loop {
            match self.create_connection(&config).await {
                Ok(()) => {
                    info!("JSON RPC link disconnected, retrying connection in {} seconds", retry_wait_seconds);
                }
                Err(e) => {
                    error!("JSON RPC link client error: {:?}", e);
                    info!("Retrying JSON RPC link connection in {} seconds", retry_wait_seconds);
                }
            }
            tokio::time::sleep(Duration::from_secs(retry_wait_seconds)).await;
            retry_wait_seconds = (retry_wait_seconds.pow(2)).min(60 * 60);
        }
    }

    async fn create_connection(
        &mut self,
        config: &JsonRpcLinkConfigClient,
    ) -> Result<(), JsonRcpLinkClientError> {
        let client = ManagerClient::connect(
            ClientConfig {
                url: config.url.clone(),
                root_certificate: self.state.config().root_certificate(),
                api_key: self.state.config().api_key().to_string(),
            }
        )   .await
            .change_context(JsonRcpLinkClientError::Client)?;

        let (reader, writer) = client
            .json_rpc_link(self.state.config().manager_name(), config.password.clone())
            .change_context(JsonRcpLinkClientError::Client)
            .await?;

        info!("JSON RPC link connected");

        let (sender, receiver) = mpsc::channel(10);

        tokio::select! {
            r = self.send_connection_tests(sender.clone()) => r,
            r = self.handle_reading(reader, sender) => r,
            r = self.handle_writing(writer, receiver) => r,
        }
    }

    async fn send_connection_tests(
        &self,
        sender: mpsc::Sender<JsonRpcLinkMessage>,
    ) -> Result<(), JsonRcpLinkClientError> {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
            sender.send(JsonRpcLinkMessage::empty())
                .await
                .change_context(JsonRcpLinkClientError::BrokenMessageChannel)?;
        }
    }

    async fn handle_reading(
        &self,
        mut reader: Box<dyn ClientConnectionRead>,
        sender: mpsc::Sender<JsonRpcLinkMessage>,
    ) -> Result<(), JsonRcpLinkClientError> {
        loop {
            let Some(m) = reader.receive_json_rpc_link_message()
                .await
                .change_context(JsonRcpLinkClientError::Read)? else {
                    return Ok(());
                };

            match m.header.message_type {
                JsonRpcLinkMessageType::Empty => (),
                JsonRpcLinkMessageType::ServerResponse =>
                    warn!("Ignoring ServerResponse message. Client should send ServerRequest messages"),
                JsonRpcLinkMessageType::ServerRequest => {
                    let request: JsonRpcRequest = serde_json::from_str(&m.data)
                        .change_context(JsonRcpLinkClientError::Deserialize)?;

                    let r = handle_rpc_request(request, Some("JSON RPC link server".to_string()), &self.state).await;
                    let r = match r {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("Request handling error: {:?}", e);
                            continue;
                        }
                    };
                    let data = serde_json::to_string(&r)
                        .change_context(JsonRcpLinkClientError::Serialize)?;
                    sender.send(JsonRpcLinkMessage {
                        header: JsonRpcLinkHeader {
                            message_type: JsonRpcLinkMessageType::ServerResponse,
                            sequence_number: m.header.sequence_number,
                        },
                        data,
                    })
                        .await
                        .change_context(JsonRcpLinkClientError::BrokenMessageChannel)?;
                }
            }
        }
    }

    async fn handle_writing(
        &self,
        mut writer: Box<dyn ClientConnectionWrite>,
        mut receiver: mpsc::Receiver<JsonRpcLinkMessage>,
    ) -> Result<(), JsonRcpLinkClientError> {
        loop {
            let message = match receiver.recv().await {
                Some(m) => m,
                None => return Err(JsonRcpLinkClientError::BrokenMessageChannel.report()),
            };
            writer.send_json_rpc_link_message(message)
                .await
                .change_context(JsonRcpLinkClientError::Write)?;
        }
    }
}
