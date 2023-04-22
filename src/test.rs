//! Run test suite and benchmarks

mod bot;
mod server;
pub mod client;

use std::{fs, sync::Arc};

use tokio::{
    select, signal,
    sync::{mpsc, watch},
};
use tracing::{error, info};

use api_client::{models::AccountIdLight};

use crate::{
    api::model::AccountId,
    config::{args::TestMode, Config},
    server::database::DB_HISTORY_DIR_NAME,
    test::{bot::{BotManager}, server::ServerManager, client::ApiClient},
};

pub struct TestRunner {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
}

impl TestRunner {
    pub fn new(config: Config, test_config: TestMode) -> Self {
        Self {
            config: config.into(),
            test_config: test_config.into(),
        }
    }

    pub async fn run(self) {
        tracing_subscriber::fmt::init();

        info!("Testing mode");

        ApiClient::new(self.test_config.server.api_urls.clone()).print_to_log();

        let server = ServerManager::new(&self.test_config).await;

        let (bot_running_handle, mut wait_all_bots) = mpsc::channel(1);
        let (quit_handle, bot_quit_receiver) = watch::channel(());

        let mut task_number = 0;
        let _api_urls = Arc::new(self.test_config.server.api_urls.clone());

        info!(
            "Task count: {}, Bot count per task: {}",
            self.test_config.task_count,
            self.test_config.bot_count,
        );

        while task_number < self.test_config.task_count {
            BotManager::spawn(
                task_number,
                self.test_config.clone(),
                None,
                bot_quit_receiver.clone(),
                bot_running_handle.clone(),
            );
            task_number += 1;
        }

        info!(
            "Bot tasks are now created",
        );

        drop(bot_running_handle);
        drop(bot_quit_receiver);


        select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(()) => (),
                    Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
                }
            }
            _ = wait_all_bots.recv() => ()
        }

        drop(quit_handle); // Singnal quit to bots.

        // Wait that all bot_running_handles are dropped.
        loop {
            match wait_all_bots.recv().await {
                None => break,
                Some(()) => (),
            }
        }

        // Quit
        server.close().await;
    }
}
