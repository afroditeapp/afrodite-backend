pub mod api;
pub mod config;
pub mod server;
pub mod utils;

use server::PihkaServer;

fn main() {
    let config = config::get_config().unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async { PihkaServer::new(config).run().await })
}
