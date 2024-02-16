//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::{Path, State};
use model::{AccountId, BooleanSetting};
use simple_backend::create_counters;

use super::super::app::{
    GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData, WriteData,
};
use crate::{api::utils::StatusCode, internal_api};


create_counters!(
    ProfileInternalCounters,
    PROFILE_INTERNAL,
    PROFILE_INTERNAL_COUNTERS_LIST,
);
