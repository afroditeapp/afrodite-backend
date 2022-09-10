pub mod read;
pub mod write;

use std::{path::{Path, PathBuf}, fmt};

use tokio::sync::oneshot;

use crate::api::core::user::{RegisterBody, RegisterResponse, LoginBody, LoginResponse};

use super::{DatabeseEntryId, git::GitDatabase};
