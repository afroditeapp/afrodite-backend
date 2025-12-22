use std::{env, path::PathBuf};

use config::{GetConfigError, args::ConfigMode, get_config};
use server_data::index::info::LocationIndexInfoCreator;
use simple_backend_config::args::ServerMode;

pub fn handle_config_tools(mode: ConfigMode) -> Result<(), GetConfigError> {
    match mode {
        ConfigMode::Check { dir } => handle_check_and_view(dir, false),
        ConfigMode::View { dir } => handle_check_and_view(dir, true),
        ConfigMode::IndexInfo { dir } => handle_index_info(dir),
    }
}

fn handle_check_and_view(dir: Option<PathBuf>, print: bool) -> Result<(), GetConfigError> {
    if let Some(dir) = dir {
        env::set_current_dir(dir).unwrap();
    }

    let dir = env::current_dir().unwrap();
    let mut config_file_found = false;

    if dir.join(config::file::CONFIG_FILE_NAME).exists() {
        let c = get_config(ServerMode::default(), String::new(), String::new(), false).unwrap();

        if print {
            println!("{:#?}", c.parsed_files())
        } else {
            println!("Config loaded correctly");
        }

        config_file_found = true;
    }

    let manager_config_file = dir.join(manager_config::file::CONFIG_FILE_NAME);
    if manager_config_file.exists() {
        let c = manager_config::get_config(
            manager_config_file,
            String::new(),
            String::new(),
            String::new(),
        )
        .unwrap();

        if print {
            println!("{:#?}", c.parsed_file())
        } else {
            println!("Manager config loaded correctly");
        }

        config_file_found = true;
    }

    if !config_file_found {
        println!(
            "Could not find {} or {}",
            config::file::CONFIG_FILE_NAME,
            manager_config::file::CONFIG_FILE_NAME,
        )
    }

    Ok(())
}

fn handle_index_info(dir: Option<PathBuf>) -> Result<(), GetConfigError> {
    if let Some(dir) = dir {
        env::set_current_dir(dir).unwrap();
    }

    env::current_dir().unwrap();

    let config = get_config(ServerMode::default(), String::new(), String::new(), false).unwrap();

    println!(
        "{}",
        LocationIndexInfoCreator::new(config.location().clone()).create_all()
    );

    Ok(())
}
