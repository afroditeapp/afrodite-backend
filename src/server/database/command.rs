pub mod read;
pub mod write;

use std::{
    fmt,
    path::{Path, PathBuf},
};

use tokio::sync::oneshot;

use crate::api::core::user::{LoginBody, LoginResponse, RegisterBody, RegisterResponse};

use super::{git::GitDatabase, DatabeseEntryId};
