
use std::{future::Future, path::Path, sync::Arc};

use error_stack::{report, ResultExt};
use futures::FutureExt;
use manager_model::{JsonRpcRequest, JsonRpcRequestType, JsonRpcResponse, JsonRpcResponseType, ManagerInstanceName, ManagerInstanceNameList, ManagerProtocolMode, ManagerProtocolVersion, SecureStorageEncryptionKey, ServerEvent, SystemInfo};
use tokio::net::TcpStream;
use tokio_rustls::{rustls::pki_types::{pem::PemObject, CertificateDer, ServerName}, TlsConnector};
use url::Url;


use error_stack::Result;

use crate::config::Config;

use super::server::{json_rpc::handle_request_type, ClientConnectionReadWrite, ConnectionUtilsRead, ConnectionUtilsWrite};

pub use tokio_rustls::rustls::RootCertStore;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Write error")]
    Write,
    #[error("Read error")]
    Read,
    #[error("Unsupported scheme")]
    UnsupportedScheme,
    #[error("Url host part is missing")]
    UrlHostMissing,
    #[error("Url host part is invalid")]
    UrlHostInvalid,
    #[error("Url port is missing")]
    UrlPortMissing,
    #[error("Connecting failed")]
    Connect,
    #[error("Root certificate is not configured")]
    RootCertificateIsNotConfigured,
    #[error("Root certificate loading error")]
    RootCertificateLoadingError,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Remote API request failed")]
    RemoteApiRequest,
    #[error("Local API request failed")]
    LocalApiRequest,

    #[error("Missing configuration")]
    MissingConfiguration,
}


#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub url: Url,
    /// Required for TLS connections
    pub root_certificate: Option<RootCertStore>,
    pub api_key: String,
}

pub struct ManagerClient {
    stream: Box<dyn ClientConnectionReadWrite>,
}


impl ManagerClient {
    pub fn load_root_certificate(root_certificate: impl AsRef<Path>) -> Result<RootCertStore, ClientError> {
        let certificate = CertificateDer::from_pem_file(root_certificate)
            .change_context(ClientError::RootCertificateLoadingError)?;

        let mut root_store = RootCertStore::empty();
        root_store.add( certificate)
            .change_context(ClientError::RootCertificateLoadingError)?;

        Ok(root_store)
    }

    pub async fn connect(config: ClientConfig) -> Result<Self, ClientError> {
        let host = config.url.host_str()
            .map(|v| v.to_string())
            .ok_or_else(|| report!(ClientError::UrlHostMissing))?;
        let port = config.url.port()
            .ok_or_else(|| report!(ClientError::UrlPortMissing))?;
        match config.url.scheme() {
            "tcp" => Self::connect_tcp(config, (host, port)).await,
            "tls" => Self::connect_tls(config, (host, port)).await,
            other => Err(report!(ClientError::UnsupportedScheme))
                .attach_printable(other.to_string()),
        }
    }

    async fn connect_tcp(config: ClientConfig, host_and_port: (String, u16)) -> Result<Self, ClientError> {
        let stream = TcpStream::connect(host_and_port)
            .await
            .change_context(ClientError::Connect)?;

        Self::init_connection(config, Box::new(stream)).await
    }

    async fn connect_tls(config: ClientConfig, host_and_port: (String, u16)) -> Result<Self, ClientError> {
        let domain = ServerName::try_from(host_and_port.0.clone())
            .change_context(ClientError::UrlHostInvalid)?;

        let Some(root_store) = config.root_certificate.clone() else {
            return Err(report!(ClientError::RootCertificateIsNotConfigured));
        };

        let tls_config = tokio_rustls::rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let stream = TcpStream::connect(host_and_port)
            .await
            .change_context(ClientError::Connect)?;
        let connector = TlsConnector::from(Arc::new(tls_config));
        let stream = connector.connect(domain, stream)
            .await
            .change_context(ClientError::Connect)?;

        Self::init_connection(config, Box::new(stream)).await
    }

    async fn init_connection(
        config: ClientConfig,
        mut stream: Box<dyn ClientConnectionReadWrite>
    ) -> Result<Self, ClientError> {
        stream.send_u8(ManagerProtocolVersion::V1 as u8)
            .await
            .change_context(ClientError::Write)?;
        stream.send_string_with_u32_len(config.api_key)
            .await
            .change_context(ClientError::Write)?;
        let result = stream.receive_u8()
            .await
            .change_context(ClientError::Read)?;
        if result != 1 {
            return Err(report!(ClientError::InvalidApiKey));
        }

        Ok(ManagerClient {
            stream,
        })
    }

    pub async fn send_request(
        mut self,
        request: JsonRpcRequest
    ) -> Result<JsonRpcResponse, ClientError> {
        self.send_request_internal(request).await
    }

    async fn send_request_internal(
        &mut self,
        request: JsonRpcRequest
    ) -> Result<JsonRpcResponse, ClientError> {
        self.stream.send_u8(ManagerProtocolMode::JsonRpc as u8)
            .await
            .change_context(ClientError::Write)?;
        self.stream.send_json_rpc_request(request)
            .await
            .change_context(ClientError::Write)?;
        self.stream.receive_json_rpc_response()
            .await
            .change_context(ClientError::Write)
    }

    pub async fn listen_events(
        mut self,
    ) -> Result<ServerEventListerner, ClientError> {
        self.stream.send_u8(ManagerProtocolMode::ListenServerEvents as u8)
            .await
            .change_context(ClientError::Write)?;
        Ok(ServerEventListerner { stream: self.stream })
    }

    pub fn request_to(self, name: ManagerInstanceName) -> ManagerClientWithRequestReceiver {
        ManagerClientWithRequestReceiver {
            client: self,
            name,
        }
    }
}

pub struct ServerEventListerner {
    stream: Box<dyn ClientConnectionReadWrite>,
}

impl ServerEventListerner {
    pub async fn next_event(&mut self) -> Result<ServerEvent, ClientError> {
        self.stream.receive_server_event()
            .await
            .change_context(ClientError::Read)
    }
}

pub trait RequestSendingSupport {
    fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Box<dyn Future<Output = Result<JsonRpcResponse, ClientError>> + Send + '_>;
}

impl RequestSendingSupport for ManagerClient {
    fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Box<dyn Future<Output = Result<JsonRpcResponse, ClientError>> + Send + '_> {
        Box::new(ManagerClient::send_request_internal(self, request).boxed())
    }
}

pub struct LocalOrRemoteApiClient<'a> {
    config: &'a Config,
    name: ManagerInstanceName,
}

impl<'a> LocalOrRemoteApiClient<'a> {
    pub fn new(config: &'a Config, request_receiver: ManagerInstanceName) -> Self {
        Self {
            config,
            name: request_receiver,
        }
    }

    async fn handle_api_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        if self.config.manager_name() == request.receiver {
            handle_request_type(
                request.request,
                self.config,
            )
                .await
                .change_context(ClientError::LocalApiRequest)
        } else if let Some(m) = self.config.find_remote_manager(&request.receiver)  {
            let config = ClientConfig {
                url: m.url.clone(),
                root_certificate: self.config.root_certificate(),
                api_key: self.config.api_key().to_string(),
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


pub struct ManagerClientWithRequestReceiver {
    client: ManagerClient,
    name: ManagerInstanceName,
}

pub trait RequestSenderCmds: Sized {
    fn request_receiver_name(&self) -> ManagerInstanceName;
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError>;

    async fn get_available_instances(
        self,
    ) -> Result<ManagerInstanceNameList, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetManagerInstanceNames,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::ManagerInstanceNames(info) = response.into_response() {
            Ok(info)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_secure_storage_encryption_key(
        self,
        key: ManagerInstanceName,
    ) -> Result<SecureStorageEncryptionKey, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSecureStorageEncryptionKey(key),
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SecureStorageEncryptionKey(key) = response.into_response() {
            Ok(key)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_system_info(
        self,
    ) -> Result<SystemInfo, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSystemInfo,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SystemInfo(info) = response.into_response() {
            Ok(info)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }
}

impl RequestSenderCmds for ManagerClientWithRequestReceiver {
    fn request_receiver_name(&self) -> ManagerInstanceName {
        self.name.clone()
    }
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        self.client.send_request(request).await
    }
}


impl RequestSenderCmds for LocalOrRemoteApiClient<'_> {
    fn request_receiver_name(&self) -> ManagerInstanceName {
        self.name.clone()
    }
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        self.handle_api_request(request).await
    }
}
