use std::{io::Write, path::Path};

use error_stack::{Result, ResultExt};
use model::StringResourceInternal;
use serde::Deserialize;

use crate::file::ConfigFileError;

const DEFAULT_NOTIFICATION_CONTENT: &str = r#"

# Like received

[like_received_single]
default = "Chat request received"

[like_received_multiple]
default = "Chat requests received"

# Media content moderation completed

[media_content_accepted]
default = "Image accepted"

[media_content_rejected]
default = "Image rejected"

[media_content_deleted]
default = "Image deleted"

[media_content_deleted_description]
default = "Unallowed content was detected from the image. This might be false positive detection."

# Profile string moderation completed

[profile_name_accepted]
default = "Profile name accepted"

[profile_name_rejected]
default = "Profile name rejected"

[profile_text_accepted]
default = "Profile text accepted"

[profile_text_rejected]
default = "Profile text rejected"

# Message received

[message_received_single]
default = "{} sent a message"

[message_received_multiple]
default = "{} sent messages"

# News item available

[news_item_available]
default = "News available"

# Automatic profile search completed

[automatic_profile_search_found_profiles_single]
default = "New or updated profile found"

[automatic_profile_search_found_profiles_multiple]
default = "{} new or updated profiles found"

"#;

#[derive(Debug, Deserialize)]
pub struct StringResourceWithFormatArg(StringResourceInternal);

#[derive(Debug, Default, Deserialize)]
pub struct NotificationContentFile {
    pub like_received_single: Option<StringResourceInternal>,
    pub like_received_multiple: Option<StringResourceInternal>,
    pub media_content_accepted: Option<StringResourceInternal>,
    pub media_content_rejected: Option<StringResourceInternal>,
    pub media_content_deleted: Option<StringResourceInternal>,
    pub media_content_deleted_description: Option<StringResourceInternal>,
    pub profile_name_accepted: Option<StringResourceInternal>,
    pub profile_name_rejected: Option<StringResourceInternal>,
    pub profile_text_accepted: Option<StringResourceInternal>,
    pub profile_text_rejected: Option<StringResourceInternal>,
    pub message_received_single: Option<StringResourceWithFormatArg>,
    pub message_received_multiple: Option<StringResourceWithFormatArg>,
    pub news_item_available: Option<StringResourceInternal>,
    pub automatic_profile_search_found_profiles_single: Option<StringResourceInternal>,
    pub automatic_profile_search_found_profiles_multiple: Option<StringResourceWithFormatArg>,
    #[serde(flatten)]
    pub other: toml::Table,
}

impl NotificationContentFile {
    pub fn load(
        file: impl AsRef<Path>,
        save_if_needed: bool,
    ) -> Result<NotificationContentFile, ConfigFileError> {
        let path = file.as_ref();
        if !path.exists() && save_if_needed {
            let mut new_file =
                std::fs::File::create_new(path).change_context(ConfigFileError::LoadConfig)?;
            new_file
                .write_all(DEFAULT_NOTIFICATION_CONTENT.as_bytes())
                .change_context(ConfigFileError::LoadConfig)?;
        }
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: NotificationContentFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        if let Some(key) = config.other.keys().next() {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "Notification content config file error. Unknown string resource '{key}'."
            ));
        }

        let no_args: &[&Option<StringResourceInternal>] = &[
            &config.like_received_single,
            &config.like_received_multiple,
            &config.media_content_accepted,
            &config.media_content_rejected,
            &config.media_content_deleted,
            &config.media_content_deleted_description,
            &config.profile_name_accepted,
            &config.profile_name_rejected,
            &config.profile_text_accepted,
            &config.profile_text_rejected,
            &config.news_item_available,
            &config.automatic_profile_search_found_profiles_single,
        ];

        for a in no_args
            .iter()
            .filter_map(|v| v.as_ref())
            .flat_map(|v| v.values())
        {
            if a.contains("{}") {
                return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                    "Notification content config file error. String does not support format arguments. Remove format arguments from '{a}'."
                ));
            }
        }

        let one_arg: &[&Option<StringResourceWithFormatArg>] = &[
            &config.message_received_single,
            &config.message_received_multiple,
            &config.automatic_profile_search_found_profiles_multiple,
        ];

        for a in one_arg
            .iter()
            .filter_map(|v| v.as_ref())
            .flat_map(|v| v.0.values())
        {
            if !a.contains("{}") || (a.replacen("{}", "", 1).contains("{}")) {
                return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                    "Notification content config file error. String required one placeholder string '{{}}'. Add that to string '{a}'."
                ));
            }
        }

        Ok(config)
    }

    pub fn get<'a>(&'a self, language: Option<&'a str>) -> NotificationStringGetter<'a> {
        NotificationStringGetter {
            config: self,
            language: language.unwrap_or_default(),
        }
    }
}

pub struct NotificationStringGetter<'a> {
    config: &'a NotificationContentFile,
    language: &'a str,
}

macro_rules! no_args {
    ($( ($name:ident, $default:literal),)*) => {
        impl<'a> NotificationStringGetter<'a> {
            $(
                pub fn $name(&self) -> String {
                    self.config
                        .$name
                        .as_ref()
                        .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
                        .cloned()
                        .unwrap_or($default.to_string())
                }
            )*
        }
    };
}

no_args!(
    (like_received_single, "Chat request received"),
    (like_received_multiple, "Chat requests received"),
    (media_content_accepted, "Image accepted"),
    (media_content_rejected, "Image rejected"),
    (media_content_deleted, "Image deleted"),
    (
        media_content_deleted_description,
        "Unallowed content was detected from the image. This might be false positive detection."
    ),
    (profile_name_accepted, "Profile name accepted"),
    (profile_name_rejected, "Profile name rejected"),
    (profile_text_accepted, "Profile text accepted"),
    (profile_text_rejected, "Profile text rejected"),
    (news_item_available, "News available"),
    (
        automatic_profile_search_found_profiles_single,
        "New or updated profile found"
    ),
);

macro_rules! one_arg {
    ($( ($name:ident, $default:literal),)*) => {
        impl<'a> NotificationStringGetter<'a> {
            $(
                pub fn $name(&self, arg: &str) -> String {
                    self.config
                        .$name
                        .as_ref()
                        .map(|v| v.0.translations.get(self.language).unwrap_or(&v.0.default))
                        .cloned()
                        .unwrap_or($default.to_string())
                        .replace("{}", &arg)
                }
            )*
        }
    };
}

one_arg!(
    (message_received_single, "{} sent a message"),
    (message_received_multiple, "{} sent messages"),
    (
        automatic_profile_search_found_profiles_multiple,
        "{} new or updated profiles found"
    ),
);
