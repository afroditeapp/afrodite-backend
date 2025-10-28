use std::{collections::HashMap, io::Write, path::Path};

use error_stack::{Result, ResultExt};
use handlebars::Handlebars;
use model::StringResourceInternal;
use serde::Deserialize;
use serde_json::json;

use crate::file::ConfigFileError;

const DEFAULT_EMAIL_CONTENT: &str = r#"
# Common template for all emails (non-translatable, required)
# All custom keys plus "subject" and "body" are available in the template
email_body_template = """
{{subject}}

{{body}}

{{footer}}
"""

email_body_content_type_is_html = false

[custom_keys.footer]
default = "Footer"

# Email confirmation

[email_confirmation.subject]
default = "Confirm your email address"

[email_confirmation.body]
default = "Please confirm your email address by opening this link: https://example.com/account_api/confirm_email_address/{{token}}"

# New message

[new_message.subject]
default = "New message received"

[new_message.body]
default = "You have received a new message"

# New like

[new_like.subject]
default = "New chat request received"

[new_like.body]
default = "You have received a new chat request"

# Account deletion remainder 1/3

[account_deletion_remainder_first.subject]
default = "Account deletion reminder 1/3"

[account_deletion_remainder_first.body]
default = "Your account will be deleted. This is the first reminder."

# Account deletion remainder 2/3

[account_deletion_remainder_second.subject]
default = "Account deletion reminder 2/3"

[account_deletion_remainder_second.body]
default = "Your account will be deleted. This is the second reminder."

# Account deletion remainder 3/3

[account_deletion_remainder_third.subject]
default = "Account deletion reminder 3/3"

[account_deletion_remainder_third.body]
default = "Your account will be deleted. This is the final reminder."

"#;

#[derive(Debug, Clone)]
pub struct EmailContent {
    pub subject: String,
    pub body: String,
    pub body_is_html: bool,
}

#[derive(Debug, Default, Deserialize)]
struct EmailContentStrings {
    subject: StringResourceInternal,
    body: StringResourceInternal,
}

#[derive(Debug, Deserialize)]
pub struct EmailContentFile {
    email_body_template: String,
    email_body_content_type_is_html: bool,
    #[serde(default)]
    custom_keys: HashMap<String, StringResourceInternal>,
    /// "{{token}}" is replaced with email confirmation token
    email_confirmation: Option<EmailContentStrings>,
    new_message: Option<EmailContentStrings>,
    new_like: Option<EmailContentStrings>,
    account_deletion_remainder_first: Option<EmailContentStrings>,
    account_deletion_remainder_second: Option<EmailContentStrings>,
    account_deletion_remainder_third: Option<EmailContentStrings>,
    #[serde(flatten)]
    other: toml::Table,
}

impl EmailContentFile {
    pub fn load(
        file: impl AsRef<Path>,
        save_if_needed: bool,
    ) -> Result<EmailContentFile, ConfigFileError> {
        let path = file.as_ref();
        if !path.exists() && save_if_needed {
            let mut new_file =
                std::fs::File::create_new(path).change_context(ConfigFileError::LoadConfig)?;
            new_file
                .write_all(DEFAULT_EMAIL_CONTENT.as_bytes())
                .change_context(ConfigFileError::LoadConfig)?;
        }
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: EmailContentFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        if let Some(key) = config.other.keys().next() {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "Email content config file error. Unknown string resource '{key}'."
            ));
        }

        // Validate that template can be parsed
        if let Err(e) = Handlebars::new().render_template_with_context_to_write(
            &config.email_body_template,
            &handlebars::Context::null(),
            &mut std::io::sink(),
        ) {
            return Err(ConfigFileError::InvalidConfig)
                .attach_printable(format!("Template parsing error: {e}"));
        }

        // Find all variable references in the template
        let mut referenced_keys = std::collections::HashSet::new();
        for line in config.email_body_template.lines() {
            for cap in line.match_indices("{{") {
                if let Some(end_pos) = line[cap.0..].find("}}") {
                    let var_content = &line[cap.0 + 2..cap.0 + end_pos].trim();
                    // Extract variable name (handle helpers and paths)
                    let var_name = var_content.split_whitespace().next().unwrap_or("");
                    let var_name = var_name.trim_start_matches('#').trim_start_matches('/');
                    if !var_name.is_empty() && var_name != "subject" && var_name != "body" {
                        referenced_keys.insert(var_name.to_string());
                    }
                }
            }
        }

        // Check if all custom keys are referenced in the template
        for custom_key in config.custom_keys.keys() {
            if !referenced_keys.contains(custom_key) {
                return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                    "Custom key '{custom_key}' is defined but not referenced in the template",
                ));
            }
        }

        if let Some(email_confirmation) = &config.email_confirmation {
            if !email_confirmation.body.all_strings_contain("{{token}}") {
                return Err(ConfigFileError::InvalidConfig).attach_printable(
                    "'{{token}}' is missing from email_confirmation body text".to_string(),
                );
            }
        }

        Ok(config)
    }

    pub fn get<'a, T: AsRef<str>>(&'a self, language: Option<&'a T>) -> EmailStringGetter<'a> {
        EmailStringGetter {
            config: self,
            language: language.map(|v| v.as_ref()).unwrap_or_default(),
        }
    }
}

pub struct EmailStringGetter<'a> {
    config: &'a EmailContentFile,
    language: &'a str,
}

impl<'a> EmailStringGetter<'a> {
    fn apply_template(
        &self,
        resource: &Option<EmailContentStrings>,
        default_subject: &str,
        default_body: &str,
    ) -> Result<EmailContent, ConfigFileError> {
        self.render_body_and_apply_template(resource, default_subject, default_body, HashMap::new())
    }

    fn render_body_and_apply_template(
        &self,
        resource: &Option<EmailContentStrings>,
        default_subject: &str,
        default_body: &str,
        body_data: HashMap<&str, &str>,
    ) -> Result<EmailContent, ConfigFileError> {
        let subject = resource
            .as_ref()
            .map(|v| &v.subject)
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or_else(|| default_subject.to_string());

        let body = resource
            .as_ref()
            .map(|v| &v.body)
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or_else(|| default_body.to_string());

        let rendered_body = Handlebars::new()
            .render_template(&body, &body_data)
            .change_context(ConfigFileError::InvalidConfig)
            .attach_printable_lazy(|| "Template rendering error".to_string())?;

        let mut data = json!({
            "subject": subject,
            "body": rendered_body,
        });

        // Add custom keys
        for (key, resource) in &self.config.custom_keys {
            let value = resource
                .translations
                .get(self.language)
                .unwrap_or(&resource.default);
            data[key] = json!(value);
        }

        let rendered = Handlebars::new()
            .render_template(&self.config.email_body_template, &data)
            .change_context(ConfigFileError::InvalidConfig)
            .attach_printable_lazy(|| "Template rendering error".to_string())?;

        Ok(EmailContent {
            subject,
            body: rendered,
            body_is_html: self.config.email_body_content_type_is_html,
        })
    }

    pub fn email_confirmation(&self, token: &str) -> Result<EmailContent, ConfigFileError> {
        self.render_body_and_apply_template(
            &self.config.email_confirmation,
            "Confirm your email address",
            "Please confirm your email address by opening this link: https://example.com/account_api/confirm_email_address/{{token}}",
            HashMap::from_iter([("token", token)]),
        )
    }

    pub fn new_message(&self) -> Result<EmailContent, ConfigFileError> {
        self.apply_template(
            &self.config.new_message,
            "New message received",
            "You have received a new message",
        )
    }

    pub fn new_like(&self) -> Result<EmailContent, ConfigFileError> {
        self.apply_template(
            &self.config.new_like,
            "New chat request received",
            "You have received a new chat request",
        )
    }

    pub fn account_deletion_remainder_first(&self) -> Result<EmailContent, ConfigFileError> {
        self.apply_template(
            &self.config.account_deletion_remainder_first,
            "Account deletion reminder 1/3",
            "Your account will be deleted. This is the first reminder.",
        )
    }

    pub fn account_deletion_remainder_second(&self) -> Result<EmailContent, ConfigFileError> {
        self.apply_template(
            &self.config.account_deletion_remainder_second,
            "Account deletion reminder 2/3",
            "Your account will be deleted. This is the second reminder.",
        )
    }

    pub fn account_deletion_remainder_third(&self) -> Result<EmailContent, ConfigFileError> {
        self.apply_template(
            &self.config.account_deletion_remainder_third,
            "Account deletion reminder 3/3",
            "Your account will be deleted. This is the final reminder.",
        )
    }
}
