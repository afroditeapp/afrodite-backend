use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::api::{
    account::data::{AccountId, ApiKey},
    model::{AccountIdInternal, AccountIdLight, AccountState, Account},
};

use super::database::{write::WriteCommands, RouterDatabaseReadHandle, commands::WriteCommandRunnerHandle};

pub struct SessionManager {
    pub database: RouterDatabaseReadHandle,
}

impl SessionManager {
    pub async fn new(database_handle: RouterDatabaseReadHandle) -> Self {
        Self {
            database: database_handle,
        }
    }
}
