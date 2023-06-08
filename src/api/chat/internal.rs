//! Handlers for internal from Server to Server state transfers and messages

use axum::{extract::Path, Json};

use hyper::StatusCode;

use crate::api::{
    model::{AccountIdLight, BooleanSetting, Profile},
    GetInternalApi, GetUsers, ReadDatabase,
};

use tracing::{error};
