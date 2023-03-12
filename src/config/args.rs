use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
};

use clap::{arg, command, value_parser, Command};
use reqwest::Url;

use crate::client::PublicApiUrls;

use super::ClientApiUrls;

// Config given as command line arguments
pub struct ArgsConfig {
    pub database_dir: Option<PathBuf>,
    pub test_mode: Option<TestMode>,
}

pub fn get_config() -> ArgsConfig {
    let matches = command!()
        .arg(
            arg!(--database <DIR> "Set database directory. Overrides config file value.")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .subcommand(Command::new("test")
            .about("Run tests and benchmarks")
            .arg(arg!(--count <COUNT> "Bot user count")
                .value_parser(value_parser!(u32))
                .default_value("1")
                .required(false))
            .arg(arg!(--account <URL> "Base URL for account API")
                .value_parser(value_parser!(Url))
                .default_value("http://127.0.0.1:3000")
                .required(false))
            .arg(arg!(--profile <URL> "Base URL for profile API")
                .value_parser(value_parser!(Url))
                .default_value("http://127.0.0.1:3000")
                .required(false))
            .arg(arg!(--"no-sleep" "Make bots to make requests constantly"))
            .arg(arg!(--"update-profile" "Update profile continuously"))
            .arg(arg!(--"print-speed" "Print some speed information"))
            .arg(arg!(--forever "Run tests forever"))
            )
        .get_matches();

    let test_mode = match matches.subcommand() {
        Some(("test", sub_matches)) => {
            let api_urls = PublicApiUrls::new(
                sub_matches.get_one::<Url>("account").unwrap().clone(),
                sub_matches.get_one::<Url>("profile").unwrap().clone(),
            ).unwrap();

            Some(TestMode {
                bot_count: *sub_matches.get_one::<u32>("count").unwrap(),
                forever: sub_matches.is_present("forever"),
                api_urls,
                no_sleep: sub_matches.is_present("no-sleep"),
                update_profile: sub_matches.is_present("update-profile"),
                print_speed: sub_matches.is_present("print-speed"),
            })
        }
        _ => None,
    };

    ArgsConfig {
        database_dir: matches
            .get_one::<PathBuf>("database")
            .map(ToOwned::to_owned),
        test_mode,
    }
}

#[derive(Debug)]
pub enum ServerComponent {
    Login,
    Core,
    /// Run server which will serve public media files.
    Media,
}

impl TryFrom<&str> for ServerComponent {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "login" => Self::Login,
            "core" => Self::Core,
            "media" => Self::Media,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TestMode {
    pub bot_count: u32,
    pub forever: bool,
    pub api_urls: PublicApiUrls,
    pub no_sleep: bool,
    pub update_profile: bool,
    pub print_speed: bool,
}
