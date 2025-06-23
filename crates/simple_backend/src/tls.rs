use std::sync::Arc;

use futures::StreamExt;
use rustls_platform_verifier::ConfigVerifierExt;
use simple_backend_config::SimpleBackendConfig;
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_rustls::rustls::{ClientConfig, ServerConfig};
use tokio_rustls_acme::{AcmeAcceptor, AcmeConfig, caches::DirCache};
use tracing::{error, info, warn};

use crate::{HTTPS_DEFAULT_PORT, ServerQuitWatcher};

pub struct TlsManagerQuitHandle {
    handle: Option<JoinHandle<()>>,
}

impl TlsManagerQuitHandle {
    pub async fn wait_quit(self) {
        if let Some(handle) = self.handle {
            match handle.await {
                Ok(()) => (),
                Err(e) => {
                    warn!("TlsManagerQuitFailed quit failed. Error: {:?}", e);
                }
            }
        }
    }
}

pub struct TlsManager {
    config: Option<SimpleBackendTlsConfig>,
}

impl TlsManager {
    pub async fn new(
        config: &SimpleBackendConfig,
        mut quit_notification: ServerQuitWatcher,
    ) -> (Self, TlsManagerQuitHandle) {
        if let Some(tls_config) = config.public_api_tls_config() {
            let manager = Self {
                config: Some(SimpleBackendTlsConfig::ManualSertificates {
                    tls_config: tls_config.clone(),
                }),
            };
            let quit_handle = TlsManagerQuitHandle { handle: None };
            (manager, quit_handle)
        } else if let Some(lets_encrypt) = config.lets_encrypt_config() {
            let client_tls_config = ClientConfig::with_platform_verifier()
                .expect("Getting platform TLS key verifier failed");
            let mut state = AcmeConfig::new(lets_encrypt.domains.clone())
                .client_tls_config(client_tls_config.into())
                .contact([format!("mailto:{}", lets_encrypt.email)])
                .cache(DirCache::new(lets_encrypt.cache_dir.clone()))
                .directory_lets_encrypt(lets_encrypt.production_servers)
                .state();
            let acceptor = state.acceptor();
            let tls_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_cert_resolver(state.resolver())
                .into();

            let Some(mut https_addr) = config.socket().public_api else {
                panic!("Public API must be enabled when using Let's Encrypt TLS");
            };
            https_addr.set_port(HTTPS_DEFAULT_PORT);
            info!(
                "HTTPS socket for Let's Encrypt ACME challenge is available on {}",
                https_addr
            );
            let https_listener = TcpListener::bind(https_addr)
                .await
                .expect("Address not available");

            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = quit_notification.recv() => break,
                        next_state = state.next() => {
                            match next_state {
                                None => break,
                                Some(Ok(value)) => info!("ACME state updated: {:?}", value),
                                Some(Err(e)) => error!("ACME state error: {}", e),
                            }
                        }
                    }
                }
            });
            (
                Self {
                    config: Some(SimpleBackendTlsConfig::LetsEncrypt {
                        tls_config,
                        utils: Some(LetsEncryptAcmeSocketUtils {
                            acceptor,
                            https_listener,
                        }),
                    }),
                },
                TlsManagerQuitHandle {
                    handle: Some(handle),
                },
            )
        } else {
            (Self { config: None }, TlsManagerQuitHandle { handle: None })
        }
    }

    pub fn config_mut(&mut self) -> Option<&mut SimpleBackendTlsConfig> {
        self.config.as_mut()
    }
}

pub enum SimpleBackendTlsConfig {
    ManualSertificates {
        tls_config: Arc<ServerConfig>,
    },
    LetsEncrypt {
        tls_config: Arc<ServerConfig>,
        /// Changes to `None` when the first HTTPS socket is created.
        utils: Option<LetsEncryptAcmeSocketUtils>,
    },
}

impl SimpleBackendTlsConfig {
    pub fn take_acme_utils(&mut self) -> Option<LetsEncryptAcmeSocketUtils> {
        if let Self::LetsEncrypt { utils, .. } = self {
            utils.take()
        } else {
            None
        }
    }

    pub fn tls_config(&self) -> Arc<ServerConfig> {
        match self {
            Self::ManualSertificates { tls_config } | Self::LetsEncrypt { tls_config, .. } => {
                tls_config.clone()
            }
        }
    }
}

pub struct LetsEncryptAcmeSocketUtils {
    pub acceptor: AcmeAcceptor,
    pub https_listener: TcpListener,
}
