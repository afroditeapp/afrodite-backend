use std::path::{Path, PathBuf};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Deserializer};

use crate::file::ConfigFileError;

#[derive(Debug, Default, Deserialize)]
pub struct BotConfigFile {
    pub man_image_dir: Option<PathBuf>,
    pub woman_image_dir: Option<PathBuf>,
    /// Predefined user bots.
    #[serde(default)]
    pub bot: Vec<BotInstanceConfig>,
}

impl BotConfigFile {
    pub fn load(file: impl AsRef<Path>) -> Result<BotConfigFile, ConfigFileError> {
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: BotConfigFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let mut ids = std::collections::HashSet::<u16>::new();
        for bot in &config.bot {
            if let Some(age) = bot.age {
                if age < 18 || age > 99 {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "Bot ID {} age must be between 18 and 99",
                        bot.id
                    ));
                }
            }

            if ids.contains(&bot.id) {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Bot ID {} is defined more than once", bot.id));
            }

            if bot.image.is_some() {
                match bot.img_dir_gender() {
                    Gender::Man => {
                        if config.man_image_dir.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                            .attach_printable(format!("Bot ID {} has image file name configured but man image directory is not configured", bot.id));
                        }
                    }
                    Gender::Woman => {
                        if config.woman_image_dir.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                            .attach_printable(format!("Bot ID {} has image file name configured but woman image directory is not configured", bot.id));
                        }
                    }
                }
            }

            // TODO: Validate all fields?

            ids.insert(bot.id);
        }

        if let Some(img_dir) = &config.man_image_dir {
            check_imgs_exist(&config, img_dir, Gender::Man)?
        }

        if let Some(img_dir) = &config.woman_image_dir {
            check_imgs_exist(&config, img_dir, Gender::Woman)?
        }

        Ok(config)
    }
}

fn check_imgs_exist(
    config: &BotConfigFile,
    img_dir: &Path,
    gender: Gender,
) -> Result<(), ConfigFileError> {
    for bot in &config.bot {
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

#[derive(Debug, Deserialize)]
pub struct BotInstanceConfig {
    pub id: u16,
    pub age: Option<u8>,
    pub gender: Option<Gender>,
    pub name: Option<String>,
    /// Image file name.
    ///
    /// The image is loaded from directory which matches gender config.
    pub image: Option<String>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl BotInstanceConfig {
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
