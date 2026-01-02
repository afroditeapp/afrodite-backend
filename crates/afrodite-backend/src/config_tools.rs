use std::path::PathBuf;

use config::{GetConfigError, args::ConfigMode, bot_config_file::BotConfigFile, get_config};
use server_data::index::info::LocationIndexInfoCreator;
use simple_backend_config::args::ServerMode;

pub fn handle_config_tools(mode: ConfigMode) -> Result<(), GetConfigError> {
    match mode {
        ConfigMode::CheckServer { dir } => handle_check_and_view_server(dir, false),
        ConfigMode::ViewServer { dir } => handle_check_and_view_server(dir, true),
        ConfigMode::CheckManager { file } => handle_check_and_view_manager(file, false),
        ConfigMode::ViewManager { file } => handle_check_and_view_manager(file, true),
        ConfigMode::CheckBot { file } => handle_check_and_view_bot(file, false),
        ConfigMode::ViewBot { file } => handle_check_and_view_bot(file, true),
        ConfigMode::IndexInfo { dir } => handle_index_info(dir),
    }
}

fn handle_check_and_view_server(dir: PathBuf, print: bool) -> Result<(), GetConfigError> {
    let c = get_config(
        ServerMode::new_with_config_dir(dir),
        String::new(),
        String::new(),
        false,
    )
    .unwrap();

    if print {
        println!("{:#?}", c.parsed_files())
    } else {
        println!("Server config loaded correctly");
    }

    Ok(())
}

fn handle_check_and_view_manager(file: PathBuf, print: bool) -> Result<(), GetConfigError> {
    if !file.exists() {
        println!("Manager config file '{:?}' not found", file);
        return Ok(());
    }

    let c = manager_config::get_config(file, String::new(), String::new(), String::new()).unwrap();

    if print {
        println!("{:#?}", c.parsed_file())
    } else {
        println!("Manager config loaded correctly");
    }

    Ok(())
}

fn handle_check_and_view_bot(file: PathBuf, print: bool) -> Result<(), GetConfigError> {
    if !file.exists() {
        println!("Bot config file '{:?}' not found", file);
        return Ok(());
    }

    let c = BotConfigFile::load(file, false).unwrap();

    if print {
        println!("{:#?}", c)
    } else {
        println!("Bot config loaded correctly");
    }

    Ok(())
}

fn handle_index_info(dir: PathBuf) -> Result<(), GetConfigError> {
    let config = get_config(
        ServerMode::new_with_config_dir(dir),
        String::new(),
        String::new(),
        false,
    )
    .unwrap();

    println!(
        "{}",
        LocationIndexInfoCreator::new(config.location().clone()).create_all()
    );

    Ok(())
}
