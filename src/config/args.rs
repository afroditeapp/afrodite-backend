use std::{path::PathBuf, convert::{TryFrom, TryInto}};

use clap::{arg, command, value_parser};


// Config given as command line arguments
pub struct ArgsConfig {
    pub database_dir: Option<PathBuf>,
}

pub fn get_config() -> ArgsConfig {
    let matches = command!()
        .arg(
            arg!(--database <DIR> "Set database directory. Overrides config file value.")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    ArgsConfig {
        database_dir: matches.get_one::<PathBuf>("database").map(ToOwned::to_owned),
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
