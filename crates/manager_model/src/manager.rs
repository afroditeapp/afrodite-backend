use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub const BACKEND_REPOSITORY_NAME: &str = "backend";

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
pub struct ServerNameText {
    pub server: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SoftwareOptionsQueryParam {
    pub software_options: SoftwareOptions,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ToSchema, ValueEnum)]
pub enum SoftwareOptions {
    Backend,
}

impl SoftwareOptions {
    pub const BACKEND: &'static str = BACKEND_REPOSITORY_NAME;

    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Backend => BACKEND_REPOSITORY_NAME,
        }
    }
}

impl TryFrom<&str> for SoftwareOptions {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            Self::BACKEND => Self::Backend,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct DownloadTypeQueryParam {
    pub download_type: DownloadType,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum DownloadType {
    /// HTTP GET returns BuildInfo JSON.
    Info,
}

/// Reboot computer directly after software update.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct RebootQueryParam {
    pub reboot: bool,
}

/// Reset data related to some software.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ResetDataQueryParam {
    pub reset_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SoftwareInfo {
    pub current_software: Vec<BuildInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, ToSchema)]
pub struct BuildInfo {
    pub name: String,
}
