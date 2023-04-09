use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::api::{
    account::data::{AccountId, ApiKey},
    model::{AccountIdInternal, AccountIdLight, AccountState, Account},
};

use super::database::{write::WriteCommands, RouterDatabaseHandle};

pub struct SessionManager {
    pub database: RouterDatabaseHandle,
}

impl SessionManager {
    pub async fn new(database_handle: RouterDatabaseHandle) -> Self {
        Self {
            database: database_handle,
        }
    }
}