use std::path::{Path, PathBuf};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Deserializer};
use url::Url;

use crate::{args::TestMode, file::ConfigFileError};

#[derive(Debug, Default, Deserialize)]
pub struct BotConfigFile {
    pub man_image_dir: Option<PathBuf>,
    pub woman_image_dir: Option<PathBuf>,
    /// Config for user bots
    pub bot_config: BaseBotConfig,
    /// Override config for specific user bots.
    #[serde(default)]
    pub bot: Vec<BotInstanceConfig>,
    pub profile_text_moderation: Option<ProfileTextModerationConfig>,
    pub profile_content_moderation: Option<ProfileContentModerationConfig>,
}

impl BotConfigFile {
    pub fn load_if_bot_mode_or_default(file: impl AsRef<Path>, test_mode: &TestMode) -> Result<BotConfigFile, ConfigFileError> {
        if test_mode.bot_mode().is_none() {
            return Ok(BotConfigFile::default())
        }

        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let mut config: BotConfigFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let validate_common_config = |bot: &BaseBotConfig, id: Option<u16>| {
            let error_location = id.map(|v| format!("Bot ID {} config error.", v)).unwrap_or("Bot config error.".to_string());
            if let Some(age) = bot.age {
                if age < 18 || age > 99 {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "{} Age must be between 18 and 99",
                        error_location
                    ));
                }
            }

            if bot.image.is_some() {
                match bot.img_dir_gender() {
                    Gender::Man => {
                        if config.man_image_dir.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                                .attach_printable(format!("{} Image file name configured but man image directory is not configured", error_location));
                        }
                    }
                    Gender::Woman => {
                        if config.woman_image_dir.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                                .attach_printable(format!("{} Image file name configured but woman image directory is not configured", error_location));
                        }
                    }
                }
            }

            // TODO: Validate all fields?

            Ok(())
        };

        validate_common_config(&config.bot_config, None)?;

        let mut ids = std::collections::HashSet::<u16>::new();
        for bot in &config.bot {
            validate_common_config(&config.bot_config, Some(bot.id))?;

            if ids.contains(&bot.id) {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Bot ID {} is defined more than once", bot.id));
            }

            ids.insert(bot.id);
        }

        if let Some(img_dir) = &config.man_image_dir {
            check_imgs_exist(&config, img_dir, Gender::Man)?
        }

        if let Some(img_dir) = &config.woman_image_dir {
            check_imgs_exist(&config, img_dir, Gender::Woman)?
        }

        if let Some(config) = &config.profile_text_moderation {
            let count = config.user_text_template.split(ProfileTextModerationConfig::TEMPLATE_FORMAT_ARGUMENT).count();
            #[allow(clippy::comparison_chain)]
            if count > 2 {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable("Profile text moderation user text template: only one '%s' format argument is allowed");
            } else if count < 2 {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable("Profile text moderation user text template: '%s' format argument is missing");
            }
        }

        config.merge_base_bot_config_with_specific_bot_configs();

        Ok(config)
    }

    fn merge_base_bot_config_with_specific_bot_configs(&mut self) {
        for config in &mut self.bot {
            let base = self.bot_config.clone();
            let c = config.config.clone();
            config.config = BaseBotConfig {
                age: c.age.or(base.age),
                gender: c.gender.or(base.gender),
                name: c.name.or(base.name),
                image: c.image.or(base.image),
                random_color_image: c.random_color_image.or(base.random_color_image),
                grid_crop_size: c.grid_crop_size.or(base.grid_crop_size),
                grid_crop_x: c.grid_crop_x.or(base.grid_crop_x),
                grid_crop_y: c.grid_crop_y.or(base.grid_crop_y),
                send_like_to_account_id: c.send_like_to_account_id.or(base.send_like_to_account_id),
                change_visibility: c.change_visibility.or(base.random_color_image),
                change_location: c.change_location.or(base.change_location),
            };
        }
    }
}

fn check_imgs_exist(
    config: &BotConfigFile,
    img_dir: &Path,
    gender: Gender,
) -> Result<(), ConfigFileError> {
    let configs = [&config.bot_config].into_iter()
        .chain(config.bot.iter().map(|v| &v.config));

    for bot in configs {
        if bot.img_dir_gender() != gender {
            continue;
        }

        if let Some(img) = &bot.image {
            let img_path = img_dir.join(img);
            if !img_path.is_file() {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Image file {:?} does not exist", img_path));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BaseBotConfig {
    pub age: Option<u8>,
    pub gender: Option<Gender>,
    pub name: Option<String>,
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
    /// All bots will try to send like to this account ID
    pub send_like_to_account_id: Option<simple_backend_utils::UuidBase64Url>,
    change_visibility: Option<bool>,
    change_location: Option<bool>,
}

impl BaseBotConfig {
    pub fn get_img(&self, config: &BotConfigFile) -> Option<PathBuf> {
        if let Some(img) = self.image.as_ref() {
            match self.img_dir_gender() {
                Gender::Man => config.man_image_dir.as_ref().map(|dir| dir.join(img)),
                Gender::Woman => config.woman_image_dir.as_ref().map(|dir| dir.join(img)),
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
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ProfileTextModerationConfig {
    /// For example "http://localhost:11434/v1"
    pub openai_api_url: Url,
    pub model: String,
    pub system_text: String,
    /// Format argument "%s" is replaced with profile text.
    pub user_text_template: String,
    /// If LLM response starts with this text the profile text
    /// is moderated as accepted. The comparison is not case sensitive.
    pub expected_response_beginning_text: String,
    /// Accept all texts which only have single visible character.
    pub accept_single_visible_character: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
}

impl ProfileTextModerationConfig {
    pub const TEMPLATE_FORMAT_ARGUMENT: &'static str = "%s";
}

#[derive(Debug, Deserialize)]
pub struct ProfileContentModerationConfig {
    pub initial_content: bool,
    pub added_content: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
}
