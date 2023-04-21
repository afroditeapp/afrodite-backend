use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
};

use clap::{arg, command, value_parser, Command, PossibleValue};
use reqwest::Url;

use crate::test::client::PublicApiUrls;

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
        .subcommand(
            Command::new("test")
                .about("Run tests and benchmarks")
                .arg(
                    arg!(--count <COUNT> "Bot user count")
                        .value_parser(value_parser!(u32))
                        .default_value("1")
                        .required(false),
                )
                .arg(
                    arg!(--account <URL> "Base URL for account API")
                        .value_parser(value_parser!(Url))
                        .default_value("http://127.0.0.1:3000")
                        .required(false),
                )
                .arg(
                    arg!(--profile <URL> "Base URL for profile API")
                        .value_parser(value_parser!(Url))
                        .default_value("http://127.0.0.1:3000")
                        .required(false),
                )
                .arg(
                    arg!(--media <URL> "Base URL for media API")
                        .value_parser(value_parser!(Url))
                        .default_value("http://127.0.0.1:3000")
                        .required(false),
                )
                .arg(
                    arg!(--"test-database" <DIR> "Directory for test database")
                        .value_parser(value_parser!(PathBuf))
                        .default_value("tmp_databases")
                        .required(false),
                )
                .arg(arg!(--"microservice-media" "Start media API as microservice"))
                .arg(arg!(--"microservice-profile" "Start profile API as microservice"))
                .arg(arg!(--"no-sleep" "Make bots to make requests constantly"))
                .arg(arg!(--"update-profile" "Update profile continuously"))
                .arg(arg!(--"print-speed" "Print some speed information"))
                .arg(
                    arg!(--"test" <NAME> "Select custom test")
                        .value_parser(value_parser!(Test))
                        .required(false)
                        .default_value("normal"),
                )
                .arg(arg!(--forever "Run tests forever")),
        )
        .get_matches();

    let test_mode = match matches.subcommand() {
        Some(("test", sub_matches)) => {
            let api_urls = PublicApiUrls::new(
                sub_matches.get_one::<Url>("account").unwrap().clone(),
                sub_matches.get_one::<Url>("profile").unwrap().clone(),
                sub_matches.get_one::<Url>("media").unwrap().clone(),
            );

            Some(TestMode {
                bot_count: *sub_matches.get_one::<u32>("count").unwrap(),
                forever: sub_matches.is_present("forever"),
                no_sleep: sub_matches.is_present("no-sleep"),
                update_profile: sub_matches.is_present("update-profile"),
                print_speed: sub_matches.is_present("print-speed"),
                test: sub_matches
                    .get_one::<Test>("test")
                    .map(ToOwned::to_owned)
                    .unwrap(),
                server: ServerConfig {
                    api_urls,
                    test_database_dir: sub_matches
                        .get_one::<PathBuf>("test-database")
                        .map(ToOwned::to_owned)
                        .unwrap(),
                    microservice_media: sub_matches.is_present("microservice-media"),
                    microservice_profile: sub_matches.is_present("microservice-profile"),
                },
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
    pub no_sleep: bool,
    pub update_profile: bool,
    pub print_speed: bool,
    pub test: Test,
    pub server: ServerConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub api_urls: PublicApiUrls,
    pub test_database_dir: PathBuf,
    pub microservice_media: bool,
    pub microservice_profile: bool,
}

#[derive(Debug, Clone)]
pub enum Test {
    Normal,
    Default,
}

impl Test {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Default => "default",
        }
    }
}

impl TryFrom<&str> for Test {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "normal" => Self::Normal,
            "default" => Self::Default,
            _ => return Err(()),
        })
    }
}

impl clap::builder::ValueParserFactory for Test {
    type Parser = TestNameParser;
    fn value_parser() -> Self::Parser {
        TestNameParser
    }
}

#[derive(Debug, Clone)]
pub struct TestNameParser;

impl clap::builder::TypedValueParser for TestNameParser {
    type Value = Test;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        value
            .to_str()
            .ok_or(clap::Error::raw(
                clap::ErrorKind::InvalidUtf8,
                "Text was not UTF-8.",
            ))?
            .try_into()
            .map_err(|_| clap::Error::raw(clap::ErrorKind::InvalidValue, "Unknown test"))
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::PossibleValue<'static>> + '_>> {
        Some(Box::new(
            [Test::Normal, Test::Default]
                .iter()
                .map(|value| PossibleValue::new(value.as_str())),
        ))
    }
}
