use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
};

use clap::{arg, command, value_parser, Command};

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
            .about("Run tests and benchmarkss")
            .arg(arg!(--count <COUNT> "Bot user count")
                .value_parser(value_parser!(u32))
                .default_value("1")
                .required(false))
            .arg(arg!(--forever "Run tests forever"))
            )
        .get_matches();

    let test_mode = match matches.subcommand() {
        Some(("test", sub_matches)) => {
            Some(TestMode {
                bot_count: *sub_matches.get_one::<u32>("count").unwrap(),
                forever: sub_matches.is_present("forever"),
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
}
