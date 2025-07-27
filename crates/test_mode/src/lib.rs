#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::collapsible_else_if)]

//! Run test suite and benchmarks

use std::sync::Arc;

use config::{
    Config,
    args::{TestMode, TestModeSubMode},
    bot_config_file::BotConfigFile,
};
use test_mode_bot::runner::BotTestRunner;
use test_mode_tests_runner::QaTestRunner;

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
        test_mode_tests_account::call_this_to_make_sure_that_crate_is_linked();
        test_mode_tests_profile::call_this_to_make_sure_that_crate_is_linked();
        test_mode_tests_media::call_this_to_make_sure_that_crate_is_linked();

        tracing_subscriber::fmt::init();

        let reqwest_client = reqwest::Client::new();

        if let TestModeSubMode::Qa(config) = self.test_config.mode.clone() {
            QaTestRunner::new(self.config, self.test_config, config, reqwest_client)
                .run()
                .await;
        } else {
            let bot_config_file =
                if let Some(bot_config_file_path) = &self.test_config.bot_config_file {
                    match BotConfigFile::load_if_bot_mode_or_default(
                        bot_config_file_path,
                        &self.test_config,
                    ) {
                        Ok(bot_config_file) => bot_config_file,
                        Err(e) => {
                            eprintln!("Failed to load bot config file: {e:?}");
                            return;
                        }
                    }
                } else {
                    BotConfigFile::default()
                };

            BotTestRunner::new(
                self.config,
                bot_config_file.into(),
                self.test_config,
                reqwest_client,
            )
            .run()
            .await;
        }
    }
}
