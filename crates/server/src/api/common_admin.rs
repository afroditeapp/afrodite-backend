//! Common routes related to admin features

use axum::{extract::{Query, State}, Extension};
use manager_model::{
    BuildInfo, RebootQueryParam, ResetDataQueryParam, SoftwareInfo, SoftwareOptionsQueryParam,
    SystemInfoList,
};
use model::{AccountIdInternal, BackendConfig, Capabilities};
use simple_backend::{app::{GetManagerApi, PerfCounterDataProvider}, create_counters};
use simple_backend_model::{PerfHistoryQuery, PerfHistoryQueryResult};
use tracing::info;

use crate::{
    api::utils::{Json, StatusCode},
    app::{ReadData, ReadDynamicConfig, WriteDynamicConfig},
};

pub mod manager;
pub mod config;
pub mod perf;

pub use manager::*;
pub use self::config::*;
pub use perf::*;
