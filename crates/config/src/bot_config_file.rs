use std::{
    io::Write,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Deserializer};
use simple_backend_model::NonEmptyString;
pub use simple_backend_model::NsfwDetectionThresholds;
use simple_backend_utils::{
    dir::abs_path_for_directory_or_file_which_might_not_exists, time::UtcTimeValue,
};
use url::Url;

use crate::{
    args::TestMode,
    file::{ConfigFile, ConfigFileError, LocationConfig},
};

pub mod internal;

const DEFAULT_BOT_CONFIG: &str = r#"

# Local admin bot config

[content_moderation]
initial_content = true
added_content = true
default_action = "move_to_human"

[profile_name_moderation]
accept_single_visible_character = true
default_action = "move_to_human"

[profile_text_moderation]
accept_single_visible_character = true
default_action = "move_to_human"

"#;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BotConfigFile {
    #[serde(default)]
    pub image_dir: ImageDirConfig,
    /// Config for user bots
    #[serde(default)]
    pub bot_config: BaseBotConfig,
    /// Override config for specific user bots.
    #[serde(default)]
    pub bots: Vec<BotInstanceConfig>,
    pub profile_name_moderation: Option<ProfileStringModerationFileConfig>,
    pub profile_text_moderation: Option<ProfileStringModerationFileConfig>,
    pub content_moderation: Option<ContentModerationFileConfig>,
    /// Config required for starting backend in remote bot mode.
    pub remote_bot_mode: Option<RemoteBotModeConfig>,
    /// If None, reading location from server config file next
    /// to bot config file is tried.
    pub location: Option<LocationConfig>,
}

impl BotConfigFile {
    pub const CONFIG_FILE_NAME: &str = "bots.toml";
    /// Changes working directory where the config file is located if
    /// config is loaded.
    pub fn load_if_bot_mode_or_default(
        file: impl AsRef<Path>,
        test_mode: &TestMode,
    ) -> Result<BotConfigFile, ConfigFileError> {
        if test_mode.bot_mode().is_none() {
            return Ok(BotConfigFile::default());
        }

        let bot_config_abs_path = abs_path_for_directory_or_file_which_might_not_exists(file)
            .change_context(ConfigFileError::LoadConfig)?;
        let mut bot_config = Self::load(&bot_config_abs_path, false)?;

        if bot_config.location.is_none() {
            let mut server_config_path = bot_config_abs_path;
            server_config_path.pop();
            server_config_path.push(ConfigFile::CONFIG_FILE_NAME);
            let server_config = ConfigFile::load(server_config_path).attach_printable(
                "Bot config does not have [location] and server config could not be loaded. Try adding [location] to bot config.",
            )?;
            bot_config.location = server_config.location;
        }

        Ok(bot_config)
    }

    /// Changes working directory where the config file is located
    pub fn load(
        file: impl AsRef<Path>,
        save_if_needed: bool,
    ) -> Result<BotConfigFile, ConfigFileError> {
        let path = abs_path_for_directory_or_file_which_might_not_exists(file.as_ref())
            .change_context(ConfigFileError::LoadConfig)?;
        if !path.exists() && save_if_needed {
            let mut new_file =
                std::fs::File::create_new(&path).change_context(ConfigFileError::LoadConfig)?;
            new_file
                .write_all(DEFAULT_BOT_CONFIG.as_bytes())
                .change_context(ConfigFileError::LoadConfig)?;
        }
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let mut config: BotConfigFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let mut config_dir = path.clone();
        config_dir.pop();
        std::env::set_current_dir(config_dir)
            .change_context(ConfigFileError::ChangeDirectoryFailed)?;

        let validate_common_config = |bot: &BaseBotConfig, id: Option<u16>| {
            let error_location = id
                .map(|v| format!("Bot ID {v} config error."))
                .unwrap_or("Bot config error.".to_string());
            if let Some(age) = bot.age
                && (age < 18 || age > 99)
            {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("{error_location} Age must be between 18 and 99"));
            }

            if bot.image.is_some() {
                match bot.img_dir_gender() {
                    Gender::Man => {
                        if config.image_dir.man.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                                .attach_printable(format!("{error_location} Image file name configured but man image directory is not configured"));
                        }
                    }
                    Gender::Woman => {
                        if config.image_dir.woman.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                                .attach_printable(format!("{error_location} Image file name configured but woman image directory is not configured"));
                        }
                    }
                }

                if bot.random_color_image.is_some() {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "{error_location} Image and random color image can't be both set"
                    ));
                }
            }

            // TODO(future): Validate all fields?

            Ok(())
        };

        validate_common_config(&config.bot_config, None)?;

        let mut ids = std::collections::HashSet::<u16>::new();
        for bot in &config.bots {
            validate_common_config(&config.bot_config, Some(bot.id))?;

            if ids.contains(&bot.id) {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Bot ID {} is defined more than once", bot.id));
            }

            ids.insert(bot.id);
        }

        if let Some(img_dir) = &config.image_dir.man {
            check_imgs_exist(&config, img_dir, Gender::Man)?
        }

        if let Some(img_dir) = &config.image_dir.woman {
            check_imgs_exist(&config, img_dir, Gender::Woman)?
        }

        if let Some(config) = &config.content_moderation
            && let Some(config) = &config.nsfw_detection
            && !config.model_file.exists()
        {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "NSFW model file {} does not exists",
                config.model_file.display()
            ));
        }

        config.merge_base_bot_config_with_specific_bot_configs();

        Ok(config)
    }

    fn merge_base_bot_config_with_specific_bot_configs(&mut self) {
        for config in &mut self.bots {
            let base = self.bot_config.clone();
            let c = config.config.clone();

            let prevent_base_image_config = c.image.is_some() || c.random_color_image.is_some();
            let base_image = if prevent_base_image_config {
                None
            } else {
                base.image
            };
            let base_random_color_image = if prevent_base_image_config {
                None
            } else {
                base.random_color_image
            };

            config.config = BaseBotConfig {
                age: c.age.or(base.age),
                gender: c.gender.or(base.gender),
                name: c.name.or(base.name),
                text: c.text.or(base.text),
                image: c.image.or(base_image),
                random_color_image: c.random_color_image.or(base_random_color_image),
                grid_crop_size: c.grid_crop_size.or(base.grid_crop_size),
                grid_crop_x: c.grid_crop_x.or(base.grid_crop_x),
                grid_crop_y: c.grid_crop_y.or(base.grid_crop_y),
                lat: c.lat.or(base.lat),
                lon: c.lon.or(base.lon),
                send_like_to_account_id: c.send_like_to_account_id.or(base.send_like_to_account_id),
                change_visibility: c.change_visibility.or(base.random_color_image),
                change_location: c.change_location.or(base.change_location),
                change_profile_text_time: c
                    .change_profile_text_time
                    .or(base.change_profile_text_time),
            };
        }
    }

    pub fn find_bot_config(&self, task_id: u32) -> Option<&BotInstanceConfig> {
        self.bots
            .iter()
            .find(|v| Into::<u32>::into(v.id) == task_id)
    }
}

fn check_imgs_exist(
    config: &BotConfigFile,
    img_dir: &Path,
    gender: Gender,
) -> Result<(), ConfigFileError> {
    let configs = [&config.bot_config]
        .into_iter()
        .chain(config.bots.iter().map(|v| &v.config));

    for bot in configs {
        if bot.img_dir_gender() != gender {
            continue;
        }

        if let Some(img) = &bot.image {
            let img_path = img_dir.join(img);
            if !img_path.is_file() {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Image file {img_path:?} does not exist"));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ImageDirConfig {
    pub man: Option<PathBuf>,
    pub woman: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BaseBotConfig {
    pub age: Option<u8>,
    pub gender: Option<Gender>,
    pub name: Option<NonEmptyString>,
    pub text: Option<NonEmptyString>,
    /// Image file name.
    ///
    /// The image is loaded from directory which matches gender config.
    ///
    /// If this is not set and image directory is configured, then random
    /// image from the directory is used as profile image.
    pub image: Option<String>,
    /// Overrides image file configs and use randomly generated single color
    /// image as profile image.
    random_color_image: Option<bool>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
    /// Latitude
    pub lat: Option<f64>,
    /// Longitude
    pub lon: Option<f64>,
    /// All bots will try to send like to this account ID
    pub send_like_to_account_id: Option<simple_backend_utils::UuidBase64UrlToml>,
    change_visibility: Option<bool>,
    change_location: Option<bool>,
    change_profile_text_time: Option<UtcTimeValue>,
}

impl BaseBotConfig {
    pub fn get_img(&self, config: &BotConfigFile) -> Option<PathBuf> {
        if let Some(img) = self.image.as_ref() {
            match self.img_dir_gender() {
                Gender::Man => config.image_dir.man.as_ref().map(|dir| dir.join(img)),
                Gender::Woman => config.image_dir.woman.as_ref().map(|dir| dir.join(img)),
            }
        } else {
            None
        }
    }

    pub fn img_dir_gender(&self) -> Gender {
        match self.gender {
            None | Some(Gender::Man) => Gender::Man,
            Some(Gender::Woman) => Gender::Woman,
        }
    }

    pub fn random_color_image(&self) -> bool {
        self.random_color_image.unwrap_or_default()
    }

    pub fn change_visibility(&self) -> bool {
        self.change_visibility.unwrap_or_default()
    }

    pub fn change_location(&self) -> bool {
        self.change_location.unwrap_or_default()
    }

    pub fn change_profile_text_time(&self) -> Option<UtcTimeValue> {
        self.change_profile_text_time
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotInstanceConfig {
    pub id: u16,
    #[serde(flatten)]
    pub config: BaseBotConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Man,
    Woman,
}

impl<'de> Deserialize<'de> for Gender {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();

        match s.as_str() {
            "man" => Ok(Gender::Man),
            "woman" => Ok(Gender::Woman),
            _ => Err(serde::de::Error::custom("Invalid value for Gender")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProfileStringModerationFileConfig {
    pub llm: Option<LlmStringModerationFileConfig>,
    pub concurrency: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmStringModerationFileConfig {
    /// For example "http://localhost:11434/v1"
    pub openai_api_url: Url,
    pub model: String,
    #[serde(default)]
    pub debug_log_results: bool,
    /// Wait times in seconds between retry attempts. The length of this vector
    /// determines the number of retries. For example, [1, 5, 10] means 3 retries
    /// with 1, 5, and 10 seconds wait time respectively.
    #[serde(default)]
    pub retry_wait_times_in_seconds: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ContentModerationFileConfig {
    /// Neural network based detection.
    /// Actions: reject, move_to_human, accept and delete.
    pub nsfw_detection: Option<NsfwDetectionFileConfig>,
    /// Large language model based moderation.
    /// Actions: reject (can be replaced with move_to_human or ignore) and
    ///          accept (can be replaced with move_to_human or delete).
    pub llm_primary: Option<LlmContentModerationFileConfig>,
    /// The secondary LLM moderation will run if primary results with ignore
    /// action.
    pub llm_secondary: Option<LlmContentModerationFileConfig>,
    #[serde(default)]
    pub debug_log_delete: bool,
    /// Default value is 4.
    pub concurrency: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NsfwDetectionFileConfig {
    pub model_file: PathBuf,
    #[serde(default)]
    pub debug_log_results: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmContentModerationFileConfig {
    /// For example "http://localhost:11434/v1"
    pub openai_api_url: Url,
    pub model: String,
    #[serde(default)]
    pub debug_log_results: bool,
    /// Wait times in seconds between retry attempts. The length of this vector
    /// determines the number of retries. For example, [1, 5, 10] means 3 retries
    /// with 1, 5, and 10 seconds wait time respectively.
    #[serde(default)]
    pub retry_wait_times_in_seconds: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteBotModeConfig {
    pub api_url: Url,
    /// Password for remote bot login.
    pub password: Option<String>,
}
