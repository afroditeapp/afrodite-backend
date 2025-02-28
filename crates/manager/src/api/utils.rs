use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
};

use manager_model::ManagerInstanceName;

use super::GetConfig;
use crate::server::app::S;

/// If true then password has been guessed and manager API is now locked.
static API_SECURITY_LOCK: AtomicBool = AtomicBool::new(false);

#[allow(clippy::result_unit_err)]
pub fn validate_api_key(
    state: &S,
    address: SocketAddr,
    api_key: &str,
) -> Result<(), ()> {
    if API_SECURITY_LOCK.load(Ordering::Relaxed) {
        Err(())
    } else if state.config().api_key() != api_key {
        API_SECURITY_LOCK.store(true, Ordering::Relaxed);
        tracing::error!(
            "API key has been guessed. API is now locked. Guesser information, addr: {}",
            address
        );
        Err(())
    } else {
        Ok(())
    }
}

/// If true then password has been guessed and JSON RPC link is now locked.
static JSON_RPC_LINK_API_SECURITY_LOCK: AtomicBool = AtomicBool::new(false);

#[allow(clippy::result_unit_err)]
pub fn validate_json_rpc_link_login(
    state: &S,
    address: SocketAddr,
    name: &ManagerInstanceName,
    password: &str,
) -> Result<(), ()> {
    if JSON_RPC_LINK_API_SECURITY_LOCK.load(Ordering::Relaxed) {
        Err(())
    } else if let Some(config) = &state.config().json_rpc_link().server {
        if config.name != *name || config.password != password {
            API_SECURITY_LOCK.store(true, Ordering::Relaxed);
            tracing::error!(
                "JSON RPC link login has been guessed. Login is now locked. Guesser information, addr: {}",
                address
            );
            Err(())
        } else {
            Ok(())
        }
    } else {
        Err(())
    }
}
