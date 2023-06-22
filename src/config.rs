pub mod args;
pub mod file;

use std::{
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
    vec,
};

use error_stack::{IntoReport, Result, ResultExt};
use reqwest::Url;
use rustls_pemfile::{certs, rsa_private_keys};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

use crate::utils::IntoReportExt;

use self::{
    args::{TestMode},
    file::{
        Components, ConfigFile, ExternalServices, LocationConfig, SignInWithGoogleConfig,
        SocketConfig,
    },
};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Get working directory error")]
    GetWorkingDir,
    #[error("File loading failed")]
    LoadFileError,
    #[error("Load config file")]
    LoadConfig,

    // External service configuration errors
    #[error(
        "External service 'account internal' is required because account component is disabled."
    )]
    ExternalServiceAccountInternalMissing,
    #[error("External service 'media internal' is required because media component is disabled.")]
    ExternalServiceMediaInternalMissing,

    #[error("Parsing String constant to Url failed.")]
    ConstUrlParsingFailed,

    #[error("TLS config is required when debug mode is off")]
    TlsConfigMissing,
    #[error("TLS config creation error")]
    CreateTlsConfig,
}

#[derive(Debug)]
pub struct Config {
    file: ConfigFile,

    // Server related configs
    database: PathBuf,
    external_services: ExternalServices,
    client_api_urls: InternalApiUrls,
    sign_in_with_urls: SignInWithUrls,

    // Other configs
    test_mode: Option<TestMode>,

    // TLS
    public_api_tls_config: Option<Arc<ServerConfig>>,
    internal_api_tls_config: Option<Arc<ServerConfig>>,
}

impl Config {
    pub fn database_dir(&self) -> &Path {
        &self.database
    }

    pub fn components(&self) -> &Components {
        &self.file.components
    }

    pub fn socket(&self) -> &SocketConfig {
        &self.file.socket
    }

    pub fn location(&self) -> &LocationConfig {
        &self.file.location
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Swagger UI is enabled.
    /// * Internal API is available at same port as the public API.
    /// * Disabling HTTPS is possbile.
    pub fn debug_mode(&self) -> bool {
        self.file.debug.unwrap_or(false)
    }

    pub fn external_services(&self) -> &ExternalServices {
        &self.external_services
    }

    pub fn external_service_urls(&self) -> &InternalApiUrls {
        &self.client_api_urls
    }

    pub fn sign_in_with_urls(&self) -> &SignInWithUrls {
        &self.sign_in_with_urls
    }

    pub fn sign_in_with_google_config(&self) -> Option<&SignInWithGoogleConfig> {
        self.file.sign_in_with_google.as_ref()
    }

    /// Launch testing and benchmark mode instead of the server mode.
    pub fn test_mode(&self) -> Option<TestMode> {
        self.test_mode.clone()
    }

    pub fn admin_email(&self) -> &str {
        &self.file.admin_email
    }

    pub fn public_api_tls_config(&self) -> Option<&Arc<ServerConfig>> {
        self.public_api_tls_config.as_ref()
    }

    pub fn internal_api_tls_config(&self) -> Option<&Arc<ServerConfig>> {
        self.internal_api_tls_config.as_ref()
    }
}

pub fn get_config() -> Result<Config, GetConfigError> {
    let current_dir = std::env::current_dir().into_error(GetConfigError::GetWorkingDir)?;
    let mut file_config =
        file::ConfigFile::load(current_dir).change_context(GetConfigError::LoadFileError)?;
    let args_config = args::get_config();

    let database = if let Some(database) = args_config.database_dir {
        database
    } else {
        file_config.database.dir.clone()
    };

    let external_services = file_config.external_services.take().unwrap_or_default();

    let client_api_urls = create_client_api_urls(&file_config.components, &external_services)?;

    let public_api_tls_config = match file_config.tls.clone() {
        Some(tls_config) => Some(Arc::new(generate_server_config(
            tls_config.public_api_key.as_path(),
            tls_config.public_api_cert.as_path(),
        )?)),
        None => None,
    };

    let internal_api_tls_config = match file_config.tls.clone() {
        Some(tls_config) => Some(Arc::new(generate_server_config(
            tls_config.internal_api_key.as_path(),
            tls_config.internal_api_cert.as_path(),
        )?)),
        None => None,
    };

    if public_api_tls_config.is_none() && !file_config.debug.unwrap_or_default() {
        return Err(GetConfigError::TlsConfigMissing)
            .into_report()
            .attach_printable("TLS must be configured when debug mode is false");
    }

    Ok(Config {
        file: file_config,
        database,
        external_services,
        client_api_urls,
        test_mode: args_config.test_mode,
        sign_in_with_urls: SignInWithUrls::new()?,
        public_api_tls_config,
        internal_api_tls_config,
    })
}

#[derive(Debug, Clone)]
pub struct InternalApiUrls {
    pub account_base_url: Option<Url>,
    pub media_base_url: Option<Url>,
}

impl InternalApiUrls {
    pub fn new(account_base_url: Option<Url>, media_base_url: Option<Url>) -> Self {
        Self {
            account_base_url,
            media_base_url,
        }
    }
}

pub fn create_client_api_urls(
    components: &Components,
    external_services: &ExternalServices,
) -> Result<InternalApiUrls, GetConfigError> {
    let account_internal = if !components.account {
        let url = external_services
            .account_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceAccountInternalMissing)
            .into_report()?;
        Some(url.clone())
    } else {
        None
    };

    let media_internal = if !components.media && components.account {
        let url = external_services
            .media_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceMediaInternalMissing)
            .into_report()?;
        Some(url.clone())
    } else {
        None
    };

    Ok(InternalApiUrls {
        account_base_url: account_internal,
        media_base_url: media_internal,
    })
}

const GOOGLE_PUBLIC_KEY_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

#[derive(Debug, Clone)]
pub struct SignInWithUrls {
    /// Request to this should return JwkSet.
    pub google_public_keys: Url,
}

impl SignInWithUrls {
    pub fn new() -> Result<Self, GetConfigError> {
        Ok(Self {
            google_public_keys: Url::parse(GOOGLE_PUBLIC_KEY_URL)
                .into_error(GetConfigError::ConstUrlParsingFailed)?,
        })
    }
}

fn generate_server_config(
    key_path: &Path,
    cert_path: &Path,
) -> Result<ServerConfig, GetConfigError> {
    let mut key_reader =
        BufReader::new(std::fs::File::open(key_path).into_error(GetConfigError::CreateTlsConfig)?);
    let all_keys = rsa_private_keys(&mut key_reader).into_error(GetConfigError::CreateTlsConfig)?;

    let key = if let [key] = &all_keys[..] {
        PrivateKey(key.clone())
    } else if all_keys.is_empty() {
        return Err(GetConfigError::CreateTlsConfig)
            .into_report()
            .attach_printable("No key found");
    } else {
        return Err(GetConfigError::CreateTlsConfig)
            .into_report()
            .attach_printable("Only one key supported");
    };

    let mut cert_reader =
        BufReader::new(std::fs::File::open(cert_path).into_error(GetConfigError::CreateTlsConfig)?);
    let all_certs = certs(&mut cert_reader).into_error(GetConfigError::CreateTlsConfig)?;
    let cert = if let [cert] = &all_certs[..] {
        Certificate(cert.clone())
    } else if all_certs.is_empty() {
        return Err(GetConfigError::CreateTlsConfig)
            .into_report()
            .attach_printable("No cert found");
    } else {
        return Err(GetConfigError::CreateTlsConfig)
            .into_report()
            .attach_printable("Only one cert supported");
    };

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // TODO: configure at some point
        .with_single_cert(vec![cert], key)
        .into_error(GetConfigError::CreateTlsConfig)?;

    Ok(config)
}
