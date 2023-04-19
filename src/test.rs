//! Run test suite and benchmarks

mod bot;
pub mod client;

use std::{fs, sync::Arc};

use tokio::{
    select, signal,
    sync::{mpsc, watch},
};
use tracing::{error, info};

use api_client::{models::AccountIdLight, models::ApiKey};

use crate::{
    api::model::AccountId,
    config::{args::TestMode, Config},
    server::database::DB_HISTORY_DIR_NAME,
    test::bot::Bot,
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

        let (bot_running_handle, mut wait_all_bots) = mpsc::channel(1);
        let (quit_handle, bot_quit_receiver) = watch::channel(());

        let mut bot_number = 1;
        let api_urls = Arc::new(self.test_config.api_urls.clone());

        let history = self.config.database_dir().join(DB_HISTORY_DIR_NAME);

        for dir in fs::read_dir(history).expect("Getting dir iterator failed") {
            let dir = dir.expect("Dir entry reading failed");
            let id = dir
                .file_name()
                .to_str()
                .expect("Dir name contained non utf-8 bytes")
                .to_string();
            match AccountId::parse(id) {
                Ok(id) => {
                    if bot_number <= self.test_config.bot_count {
                        let id = AccountIdLight::new(id.as_uuid());
                        Bot::spawn(
                            bot_number,
                            self.test_config.clone(),
                            id,
                            bot_quit_receiver.clone(),
                            bot_running_handle.clone(),
                        );
                        bot_number += 1;
                    } else {
                        break;
                    }
                }
                Err(_) => {
                    // Not an account git directory.
                    continue;
                }
            }
        }

        // Create remaining bots
        while bot_number <= self.test_config.bot_count {
            Bot::spawn(
                bot_number,
                self.test_config.clone(),
                None,
                bot_quit_receiver.clone(),
                bot_running_handle.clone(),
            );
            bot_number += 1;
        }

        info!(
            "Bots are now created. Count: {}",
            self.test_config.bot_count
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
    }
}
