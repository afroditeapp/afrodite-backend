use std::{io::Write, path::Path};

use error_stack::{Result, ResultExt};
use model::StringResourceInternal;
use serde::Deserialize;

use crate::file::ConfigFileError;

const DEFAULT_NOTIFICATION_CONTENT: &str = r#"

# Like received

[like_received_single.title]
default = "Chat request received"

[like_received_multiple.title]
default = "Chat requests received"

# Media content moderation completed

[media_content_accepted.title]
default = "Image accepted"

[media_content_rejected.title]
default = "Image rejected"

[media_content_deleted.title]
default = "Image deleted"

[media_content_deleted.body]
default = "Unallowed content was detected from the image. This might be false positive detection."

# Profile string moderation completed

[profile_name_accepted.title]
default = "Profile name accepted"

[profile_name_rejected.title]
default = "Profile name rejected"

[profile_text_accepted.title]
default = "Profile text accepted"

[profile_text_rejected.title]
default = "Profile text rejected"

# Message received

[message_received_single.title]
default = "{} sent a message"

[message_received_multiple.title]
default = "{} sent messages"

# News item available

[news_item_available.title]
default = "News available"

# Automatic profile search completed

[automatic_profile_search_found_profiles_single.title]
default = "New or updated profile found"

[automatic_profile_search_found_profiles_multiple.title]
default = "{} new or updated profiles found"

"#;

#[derive(Debug, Clone)]
pub struct NotificationTitle {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct NotificationTitleAndBody {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NotificationContentTitle {
    pub title: StringResourceInternal,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NotificationContentTitleAndBody {
    pub title: StringResourceInternal,
    pub body: StringResourceInternal,
}

#[derive(Debug, Default, Deserialize)]
pub struct NotificationContentFile {
    pub like_received_single: Option<NotificationContentTitle>,
    pub like_received_multiple: Option<NotificationContentTitle>,
    pub media_content_accepted: Option<NotificationContentTitle>,
    pub media_content_rejected: Option<NotificationContentTitle>,
    pub media_content_deleted: Option<NotificationContentTitleAndBody>,
    pub profile_name_accepted: Option<NotificationContentTitle>,
    pub profile_name_rejected: Option<NotificationContentTitle>,
    pub profile_text_accepted: Option<NotificationContentTitle>,
    pub profile_text_rejected: Option<NotificationContentTitle>,
    pub message_received_single: Option<NotificationContentTitle>,
    pub message_received_multiple: Option<NotificationContentTitle>,
    pub news_item_available: Option<NotificationContentTitle>,
    pub automatic_profile_search_found_profiles_single: Option<NotificationContentTitle>,
    pub automatic_profile_search_found_profiles_multiple: Option<NotificationContentTitle>,
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

        // Validate title-only fields don't have format arguments
        let no_args_titles: &[&Option<NotificationContentTitle>] = &[
            &config.like_received_single,
            &config.like_received_multiple,
            &config.media_content_accepted,
            &config.media_content_rejected,
            &config.profile_name_accepted,
            &config.profile_name_rejected,
            &config.profile_text_accepted,
            &config.profile_text_rejected,
            &config.news_item_available,
            &config.automatic_profile_search_found_profiles_single,
        ];

        for resource in no_args_titles.iter().filter_map(|v| v.as_ref()) {
            for value in resource.title.values() {
                if value.contains("{}") {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "Notification content config file error. String does not support format arguments. Remove format arguments from '{value}'."
                    ));
                }
            }
        }

        // Validate title and body for media_content_deleted don't have format arguments
        if let Some(resource) = &config.media_content_deleted {
            for value in resource.title.values() {
                if value.contains("{}") {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "Notification content config file error. String does not support format arguments. Remove format arguments from '{value}'."
                    ));
                }
            }
            for value in resource.body.values() {
                if value.contains("{}") {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "Notification content config file error. String does not support format arguments. Remove format arguments from '{value}'."
                    ));
                }
            }
        }

        // Validate title fields that require exactly one format argument
        let one_arg_titles: &[&Option<NotificationContentTitle>] = &[
            &config.message_received_single,
            &config.message_received_multiple,
            &config.automatic_profile_search_found_profiles_multiple,
        ];

        for resource in one_arg_titles.iter().filter_map(|v| v.as_ref()) {
            for value in resource.title.values() {
                if !value.contains("{}") || (value.replacen("{}", "", 1).contains("{}")) {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "Notification content config file error. String requires exactly one placeholder string '{{}}'. Add that to string '{value}'."
                    ));
                }
            }
        }

        Ok(config)
    }

    pub fn get<'a, T: AsRef<str>>(
        &'a self,
        language: Option<&'a T>,
    ) -> NotificationStringGetter<'a> {
        NotificationStringGetter {
            config: self,
            language: language.map(|v| v.as_ref()).unwrap_or_default(),
        }
    }
}

pub struct NotificationStringGetter<'a> {
    config: &'a NotificationContentFile,
    language: &'a str,
}

impl<'a> NotificationStringGetter<'a> {
    fn get_title(
        &self,
        resource: &Option<NotificationContentTitle>,
        default: &str,
    ) -> NotificationTitle {
        let title = resource
            .as_ref()
            .map(|v| &v.title)
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or_else(|| default.to_string());

        NotificationTitle { title }
    }

    fn get_title_and_body(
        &self,
        resource: &Option<NotificationContentTitleAndBody>,
        default_title: &str,
        default_body: &str,
    ) -> NotificationTitleAndBody {
        let title = resource
            .as_ref()
            .map(|v| &v.title)
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or_else(|| default_title.to_string());

        let body = resource
            .as_ref()
            .map(|v| &v.body)
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or_else(|| default_body.to_string());

        NotificationTitleAndBody { title, body }
    }

    pub fn like_received_single(&self) -> NotificationTitle {
        self.get_title(&self.config.like_received_single, "Chat request received")
    }

    pub fn like_received_multiple(&self) -> NotificationTitle {
        self.get_title(
            &self.config.like_received_multiple,
            "Chat requests received",
        )
    }

    pub fn media_content_accepted(&self) -> NotificationTitle {
        self.get_title(&self.config.media_content_accepted, "Image accepted")
    }

    pub fn media_content_rejected(&self) -> NotificationTitle {
        self.get_title(&self.config.media_content_rejected, "Image rejected")
    }

    pub fn media_content_deleted(&self) -> NotificationTitleAndBody {
        self.get_title_and_body(
            &self.config.media_content_deleted,
            "Image deleted",
            "Unallowed content was detected from the image. This might be false positive detection.",
        )
    }

    pub fn profile_name_accepted(&self) -> NotificationTitle {
        self.get_title(&self.config.profile_name_accepted, "Profile name accepted")
    }

    pub fn profile_name_rejected(&self) -> NotificationTitle {
        self.get_title(&self.config.profile_name_rejected, "Profile name rejected")
    }

    pub fn profile_text_accepted(&self) -> NotificationTitle {
        self.get_title(&self.config.profile_text_accepted, "Profile text accepted")
    }

    pub fn profile_text_rejected(&self) -> NotificationTitle {
        self.get_title(&self.config.profile_text_rejected, "Profile text rejected")
    }

    pub fn message_received_single(&self, arg: &str) -> NotificationTitle {
        let title = self
            .get_title(&self.config.message_received_single, "{} sent a message")
            .title
            .replace("{}", arg);
        NotificationTitle { title }
    }

    pub fn message_received_multiple(&self, arg: &str) -> NotificationTitle {
        let title = self
            .get_title(&self.config.message_received_multiple, "{} sent messages")
            .title
            .replace("{}", arg);
        NotificationTitle { title }
    }

    pub fn news_item_available(&self) -> NotificationTitle {
        self.get_title(&self.config.news_item_available, "News available")
    }

    pub fn automatic_profile_search_found_profiles_single(&self) -> NotificationTitle {
        self.get_title(
            &self.config.automatic_profile_search_found_profiles_single,
            "New or updated profile found",
        )
    }

    pub fn automatic_profile_search_found_profiles_multiple(&self, arg: &str) -> NotificationTitle {
        let title = self
            .get_title(
                &self.config.automatic_profile_search_found_profiles_multiple,
                "{} new or updated profiles found",
            )
            .title
            .replace("{}", arg);
        NotificationTitle { title }
    }
}
