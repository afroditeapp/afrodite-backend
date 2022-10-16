use std::{path::PathBuf, convert::{TryFrom, TryInto}};

use clap::{arg, command, value_parser};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

pub struct Config {
    pub database_dir: PathBuf,
    pub mode: ServerMode,
}

pub fn get_config() -> Config {
    let matches = command!()
        .arg(
            arg!(--database <DIR> "Set database directory")
                .required(false)
                .default_value("database")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--mode <MODE> "Server mode")
                .required(false)
                .default_value("core")
                .value_parser(["core", "media"])
        )
        .get_matches();

    Config {
        database_dir: matches.get_one::<PathBuf>("database").unwrap().to_owned(),
        mode: matches.get_one::<String>("mode").unwrap().as_str().try_into().unwrap(),
    }
}


#[derive(Debug)]
pub enum ServerMode {
    Core,
    /// Run server which will serve public media files.
    Media,
}

impl TryFrom<&str> for ServerMode {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "core" => Self::Core,
            "media" => Self::Media,
            _ => return Err(()),
        })
    }
}
