//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::{Path, State};
use model::{AccessToken, Account, AccountId};
use simple_backend::create_counters;

use crate::{
    api::utils::{Json, StatusCode},
    app::{GetAccessTokens, GetAccounts, ReadData},
};

create_counters!(
    AccountInternalCounters,
    ACCOUNT_INTERNAL,
    ACCOUNT_INTERNAL_COUNTERS_LIST,
);
