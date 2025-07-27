use std::path::{Path, PathBuf};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Deserializer};
pub use simple_backend_config::file::NsfwDetectionThresholds;
use simple_backend_utils::time::UtcTimeValue;
use url::Url;

use crate::{args::TestMode, file::ConfigFileError};

#[derive(Debug, Default, Deserialize)]
pub struct BotConfigFile {
    #[serde(default)]
    pub image_dir: ImageDirConfig,
    /// Config for admin bots
    #[serde(default)]
    pub admin_bot_config: AdminBotConfig,
    /// Config for user bots
    #[serde(default)]
    pub bot_config: BaseBotConfig,
    /// Override config for specific user bots.
    #[serde(default)]
    pub bot: Vec<BotInstanceConfig>,
    pub profile_name_moderation: Option<ProfileStringModerationConfig>,
    pub profile_text_moderation: Option<ProfileStringModerationConfig>,
    pub content_moderation: Option<ContentModerationConfig>,
    /// Config required for starting backend in remote bot mode.
    /// Ignored when backend starts in test mode.
    pub remote_bot_mode: Option<RemoteBotModeConfig>,
}

impl BotConfigFile {
    pub fn load_if_bot_mode_or_default(
        file: impl AsRef<Path>,
        test_mode: &TestMode,
    ) -> Result<BotConfigFile, ConfigFileError> {
        if test_mode.bot_mode().is_none() {
            return Ok(BotConfigFile::default());
        }

        Self::load(file)
    }

    pub(crate) fn load(file: impl AsRef<Path>) -> Result<BotConfigFile, ConfigFileError> {
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let mut config: BotConfigFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let validate_common_config = |bot: &BaseBotConfig, id: Option<u16>| {
            let error_location = id
                .map(|v| format!("Bot ID {v} config error."))
                .unwrap_or("Bot config error.".to_string());
            if let Some(age) = bot.age {
                if age < 18 || age > 99 {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "{error_location} Age must be between 18 and 99"
                    ));
                }
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
        for bot in &config.bot {
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

        let profile_string_moderation_configs = [
            config.profile_name_moderation.as_ref(),
            config.profile_text_moderation.as_ref(),
        ];

        for config in profile_string_moderation_configs
            .iter()
            .flatten()
            .flat_map(|v| v.llm.as_ref())
        {
            let count = config
                .user_text_template
                .split(ProfileStringModerationConfig::TEMPLATE_PLACEHOLDER_TEXT)
                .count();
            #[allow(clippy::comparison_chain)]
            if count > 2 {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!(
                        "Profile text LLM moderation user text template: only one '{}' placeholder is allowed",
                        ProfileStringModerationConfig::TEMPLATE_PLACEHOLDER_TEXT,
                    ));
            } else if count < 2 {
                return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                    "Profile text LLM moderation user text template: '{}' placeholder is missing",
                    ProfileStringModerationConfig::TEMPLATE_PLACEHOLDER_TEXT,
                ));
            }
        }

        if let Some(config) = &config.content_moderation {
            if let Some(config) = &config.nsfw_detection {
                if !config.model_file.exists() {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "NSFW model file {} does not exists",
                        config.model_file.display()
                    ));
                }
            }
        }

        config.merge_base_bot_config_with_specific_bot_configs();

        Ok(config)
    }

    fn merge_base_bot_config_with_specific_bot_configs(&mut self) {
        for config in &mut self.bot {
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

    pub fn find_bot_config(&self, bot_id: u32) -> Option<&BotInstanceConfig> {
        self.bot.iter().find(|v| Into::<u32>::into(v.id) == bot_id)
    }
}

fn check_imgs_exist(
    config: &BotConfigFile,
    img_dir: &Path,
    gender: Gender,
) -> Result<(), ConfigFileError> {
    let configs = [&config.bot_config]
        .into_iter()
        .chain(config.bot.iter().map(|v| &v.config));

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
    pub name: Option<String>,
    pub text: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct BotInstanceConfig {
    pub id: u16,
    /// If `None` and account ID is not in saved state, register
    /// a new account.
    pub account_id: Option<String>,
    // Use remote bot login API.
    pub remote_bot_login_password: Option<String>,
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
pub struct ProfileStringModerationConfig {
    /// Accept all texts which only have single visible character.
    pub accept_single_visible_character: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
    /// Large language model based moderation.
    /// Actions: reject (or move_to_human) and accept
    pub llm: Option<LlmStringModerationConfig>,
    pub default_action: ModerationAction,
}

#[derive(Debug, Deserialize)]
pub struct LlmStringModerationConfig {
    /// For example "http://localhost:11434/v1"
    pub openai_api_url: Url,
    pub model: String,
    pub system_text: String,
    /// Placeholder "{text}" is replaced with text which will be
    /// moderated.
    pub user_text_template: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the profile text
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    pub move_rejected_to_human_moderation: bool,
    #[serde(default)]
    pub debug_show_llm_output_when_rejected: bool,
    #[serde(default)]
    pub debug_log_results: bool,
    #[serde(default = "max_tokens_default_value")]
    pub max_tokens: u32,
}

fn max_tokens_default_value() -> u32 {
    10_000
}

impl ProfileStringModerationConfig {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAction {
    Accept,
    Reject,
    MoveToHuman,
}

#[derive(Debug, Deserialize)]
pub struct ContentModerationConfig {
    pub initial_content: bool,
    pub added_content: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
    /// Neural network based detection.
    /// Actions: reject, move_to_human, accept and delete.
    pub nsfw_detection: Option<NsfwDetectionConfig>,
    /// Large language model based moderation.
    /// Actions: reject (can be replaced with move_to_human or ignore) and
    ///          accept (can be replaced with move_to_human or delete).
    pub llm_primary: Option<LlmContentModerationConfig>,
    /// The secondary LLM moderation will run if primary results with ignore
    /// action.
    pub llm_secondary: Option<LlmContentModerationConfig>,
    pub default_action: ModerationAction,
    #[serde(default)]
    pub debug_log_delete: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NsfwDetectionConfig {
    pub model_file: PathBuf,
    /// Thresholds for image rejection.
    pub reject: Option<NsfwDetectionThresholds>,
    /// Thresholds for moving image to human moderation.
    pub move_to_human: Option<NsfwDetectionThresholds>,
    /// Thresholds for accepting the image.
    pub accept: Option<NsfwDetectionThresholds>,
    /// Thresholds for image deletion.
    pub delete: Option<NsfwDetectionThresholds>,
    #[serde(default)]
    pub debug_log_results: bool,
}

#[derive(Debug, Deserialize)]
pub struct LlmContentModerationConfig {
    /// For example "http://localhost:11434/v1"
    pub openai_api_url: Url,
    pub model: String,
    pub system_text: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the content
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    /// Overrides [Self::move_rejected_to_human_moderation]
    pub ignore_rejected: bool,
    /// Overrides [Self::move_accepted_to_human_moderation]
    pub delete_accepted: bool,
    pub move_accepted_to_human_moderation: bool,
    pub move_rejected_to_human_moderation: bool,
    #[serde(default)]
    pub debug_show_llm_output_when_rejected: bool,
    #[serde(default)]
    pub debug_log_results: bool,
    #[serde(default = "max_tokens_default_value")]
    pub max_tokens: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AdminBotConfig {
    /// If `None` and account ID is not in saved state, register
    /// a new account.
    pub account_id: Option<String>,
    // Use remote bot login API.
    pub remote_bot_login_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoteBotModeConfig {
    pub api_url: Url,
}
