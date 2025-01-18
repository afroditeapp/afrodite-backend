use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
};

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
