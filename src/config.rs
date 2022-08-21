use std::path::PathBuf;

use clap::{value_parser, arg, command};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

pub struct Config {
    pub database_dir: PathBuf,
}

pub fn get_config() -> Config {
    let matches = command!()
        .arg(
            arg!(--database <DIR> "Set database directory")
            .required(false)
            .default_value("database")
            .value_parser(value_parser!(PathBuf))
        )
        .get_matches();

    Config {
        database_dir: matches.get_one::<PathBuf>("database").unwrap().to_owned(),
    }
}
