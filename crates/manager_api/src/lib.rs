#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

#![allow(
    async_fn_in_trait,
)]

use std::{future::Future, path::Path, sync::Arc};

use error_stack::{report, ResultExt};
use futures::FutureExt;
use manager_model::{JsonRpcRequest, JsonRpcResponse, ManagerInstanceName, ManagerProtocolMode, ManagerProtocolVersion, ServerEvent};
use protocol::{ClientConnectionRead, ClientConnectionReadWrite, ClientConnectionWrite, ConnectionUtilsRead, ConnectionUtilsWrite};
use tokio::net::TcpStream;
use tokio_rustls::{rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer, ServerName}, TlsConnector};
use url::Url;

use error_stack::Result;

pub mod protocol;
pub mod backup;

pub use protocol::{ManagerClientWithRequestReceiver, RequestSenderCmds};
pub use tokio_rustls::rustls::RootCertStore;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Write error")]
    Write,
    #[error("Read error")]
    Read,
    #[error("Flush error")]
    Flush,
    #[error("Parsing error")]
    Parse,
    #[error("Serializing error")]
    Serialize,
    #[error("Unsupported string length")]
    UnsupportedStringLength,
    #[error("Unsupported data size")]
    UnsupportedDataSize,
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
    #[error("Client authentication certificate loading error")]
    ClientAuthenticationCertificate,
    #[error("Client authentication certificate private key loading error")]
    ClientAuthenticationCertificatePrivateKey,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Invalid login")]
    InvalidLogin,
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Remote API request failed")]
    RemoteApiRequest,
    #[error("Local API request failed")]
    LocalApiRequest,
    #[error("JSON RPC link related error")]
    JsonRpcLink,
    #[error("JSON RPC related error")]
    JsonRpc,
    #[error("Timeout")]
    Timeout,

    #[error("Missing configuration")]
    MissingConfiguration,
    #[error("Invalid configuration")]
    InvalidConfiguration,
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    tls_config: Arc<tokio_rustls::rustls::ClientConfig>,
}

impl TlsConfig {
    pub fn new(
        root_certificate: impl AsRef<Path>,
        server_certificate: impl AsRef<Path>,
        server_certificate_private_key: impl AsRef<Path>,
    ) -> Result<Self, ClientError> {
        let certificate = CertificateDer::from_pem_file(root_certificate)
            .change_context(ClientError::RootCertificateLoadingError)?;

        let mut root_store = RootCertStore::empty();
        root_store.add( certificate)
            .change_context(ClientError::RootCertificateLoadingError)?;

        let certificate_iter = CertificateDer::pem_file_iter(server_certificate)
            .change_context(ClientError::ClientAuthenticationCertificate)?;
        let mut client_auth_cert_chain = vec![];
        for c in certificate_iter {
            let c = c.change_context(ClientError::ClientAuthenticationCertificate)?;
            client_auth_cert_chain.push(c);
        }

        let client_auth_private_key = PrivateKeyDer::from_pem_file(server_certificate_private_key)
            .change_context(ClientError::ClientAuthenticationCertificatePrivateKey)?;

        let tls_config = tokio_rustls::rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_auth_cert_chain, client_auth_private_key)
            .change_context(ClientError::ClientAuthenticationCertificatePrivateKey)?
            .into();

        Ok(Self {
            tls_config,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub url: Url,
    /// Required for TLS connections
    pub tls_config: Option<TlsConfig>,
    pub api_key: String,
}

pub struct ManagerClient {
    reader: Box<dyn ClientConnectionRead>,
    writer: Box<dyn ClientConnectionWrite>,
}

impl ManagerClient {
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

        let Some(tls_config) = config.tls_config.clone() else {
            return Err(report!(ClientError::RootCertificateIsNotConfigured));
        };

        let stream = TcpStream::connect(host_and_port)
            .await
            .change_context(ClientError::Connect)?;
        let connector = TlsConnector::from(tls_config.tls_config.clone());
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

        let (reader, writer) = tokio::io::split(stream);

        Ok(ManagerClient {
            reader: Box::new(reader),
            writer: Box::new(writer),
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
        self.writer.send_u8(ManagerProtocolMode::JsonRpc as u8)
            .await
            .change_context(ClientError::Write)?;
        self.writer.send_json_rpc_request(request)
            .await
            .change_context(ClientError::Write)?;
        self.reader.receive_json_rpc_response()
            .await
            .change_context(ClientError::Write)
    }

    pub async fn listen_events(
        mut self,
    ) -> Result<ServerEventListerner, ClientError> {
        self.writer.send_u8(ManagerProtocolMode::ListenServerEvents as u8)
            .await
            .change_context(ClientError::Write)?;
        Ok(ServerEventListerner { reader: self.reader })
    }

    pub fn request_to(
        self,
        request_receiver: ManagerInstanceName
    ) -> ManagerClientWithRequestReceiver {
        ManagerClientWithRequestReceiver {
            client: self,
            request_receiver,
        }
    }

    pub async fn json_rpc_link(
        mut self,
        name: ManagerInstanceName,
        password: String,
    ) -> Result<(Box<dyn ClientConnectionRead>, Box<dyn ClientConnectionWrite>), ClientError> {
        self.writer.send_u8(ManagerProtocolMode::JsonRpcLink as u8)
            .await
            .change_context(ClientError::Write)?;
        self.writer.send_string_with_u32_len(name.0)
            .await
            .change_context(ClientError::Write)?;
        self.writer.send_string_with_u32_len(password)
            .await
            .change_context(ClientError::Write)?;
        let result = self.reader.receive_u8()
            .await
            .change_context(ClientError::Read)?;
        if result != 1 {
            return Err(report!(ClientError::InvalidLogin));
        }

        Ok((self.reader, self.writer))
    }

    pub async fn backup_link(
        mut self,
        password: String,
    ) -> Result<(Box<dyn ClientConnectionRead>, Box<dyn ClientConnectionWrite>), ClientError> {
        self.writer.send_u8(ManagerProtocolMode::BackupLink as u8)
            .await
            .change_context(ClientError::Write)?;
        self.writer.send_string_with_u32_len(password)
            .await
            .change_context(ClientError::Write)?;
        let result = self.reader.receive_u8()
            .await
            .change_context(ClientError::Read)?;
        if result != 1 {
            return Err(report!(ClientError::InvalidLogin));
        }

        Ok((self.reader, self.writer))
    }
}

pub struct ServerEventListerner {
    reader: Box<dyn ClientConnectionRead>,
}

impl ServerEventListerner {
    pub async fn next_event(&mut self) -> Result<ServerEvent, ClientError> {
        self.reader.receive_server_event()
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
