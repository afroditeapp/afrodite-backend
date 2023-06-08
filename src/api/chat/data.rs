use serde::{Deserialize, Serialize};

use sqlx::Encode;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::model::{AccountIdInternal, AccountIdLight};
