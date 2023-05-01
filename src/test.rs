//! Run test suite and benchmarks

mod bot;
pub mod client;
mod server;

use std::{sync::Arc, time::Duration};

use api_client::{manual_additions, apis::configuration::Configuration};
use tokio::{
    select, signal,
    sync::{mpsc, watch},
};
use tracing::{error, info};

use crate::{
    config::{args::TestMode, Config},
    test::{bot::BotManager, client::ApiClient, server::ServerManager},
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

        let server = ServerManager::new(self.test_config.clone()).await;

        let (bot_running_handle, mut wait_all_bots) = mpsc::channel(1);
        let (quit_handle, bot_quit_receiver) = watch::channel(());

        let mut task_number = 0;
        let api_urls = Arc::new(self.test_config.server.api_urls.clone());

        info!(
            "Waiting API availability..."
        );

        let quit_now = select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(()) => true,
                    Err(e) => {
                        error!("Failed to listen CTRL+C. Error: {}", e);
                        true
                    }
                }
            }
            _ = wait_that_servers_start(ApiClient::new(api_urls.as_ref().clone())) => {
                false
            },
        };

        if !quit_now {
            info!(
                "...API ready"
            );

            info!(
                "Task count: {}, Bot count per task: {}",
                self.test_config.task_count, self.test_config.bot_count,
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

            info!("Bot tasks are now created",);
        }

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


async fn wait_that_servers_start(api: ApiClient) {
    check_api(api.account()).await;
    check_api(api.profile()).await;
    check_api(api.media()).await;
}

async fn check_api(config: &Configuration) {
    loop {
        match manual_additions::api_available(config).await {
            Ok(()) => break,
            Err(()) => (),
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
