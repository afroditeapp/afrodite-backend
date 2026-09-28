use std::{collections::HashMap, io::Write, path::Path};

use error_stack::ResultExt;
use model::StringResourceInternal;
use serde::Deserialize;
use simple_backend_utils::{Result, render_template};

use crate::file::ConfigFileError;

const DEFAULT_EMAIL_CONTENT: &str = r#"
# Common template for all emails (non-translatable, required).
# All custom keys plus "subject" and "body" are available in the template.
# Each email message can override the fields after [email].
[email]
template = """
{subject}

{body}

{footer}
"""
content_type_is_html = false

[email.custom_keys.footer]
default = "This is automatic message sent by a dating app."

# Email verification

[email_verification.subject]
default = "Verify your email address"

[email_verification.body]
default = "Please verify your email address by opening this link: https://example.com/verify_email?token={token}"

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

# Email change verification

[email_change_verification.subject]
default = "Verify your new email address"

[email_change_verification.body]
default = "Please verify your new email address by opening this link: https://example.com/verify_new_email?token={token}"

# Email change notification

[email_change_notification.subject]
default = "Email change notification"

[email_change_notification.body]
default = "Your account's email address will be changed. If you did not request this change, your account might have been compromised."

# Email login token

[email_login.subject]
default = "Your login code"

[email_login.body]
default = "Here is your login code: {token}"

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
    #[serde(default, flatten)]
    email_config: Option<EmailConfigOptional>,
}

#[derive(Debug, Deserialize)]
pub struct EmailContentFile {
    email: EmailConfig,
    /// "{token}" is replaced with email verification token
    email_verification: Option<EmailContentStrings>,
    new_message: Option<EmailContentStrings>,
    new_like: Option<EmailContentStrings>,
    account_deletion_remainder_first: Option<EmailContentStrings>,
    account_deletion_remainder_second: Option<EmailContentStrings>,
    account_deletion_remainder_third: Option<EmailContentStrings>,
    /// "{token}" is replaced with email change verification token
    email_change_verification: Option<EmailContentStrings>,
    email_change_notification: Option<EmailContentStrings>,
    /// "{token}" is replaced with email login token
    email_login: Option<EmailContentStrings>,
    #[serde(flatten)]
    other: toml::Table,
}

#[derive(Debug, Deserialize)]
struct EmailConfig {
    template: String,
    content_type_is_html: bool,
    #[serde(default)]
    custom_keys: HashMap<String, StringResourceInternal>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct EmailConfigOptional {
    template: Option<String>,
    content_type_is_html: Option<bool>,
    custom_keys: Option<HashMap<String, StringResourceInternal>>,
}

const DEFAULT_EMAIL_TEMPLATE: &str = "
{subject}

{body}
";

impl Default for EmailContentFile {
    fn default() -> Self {
        Self {
            email: EmailConfig {
                template: DEFAULT_EMAIL_TEMPLATE.to_string(),
                content_type_is_html: false,
                custom_keys: HashMap::new(),
            },
            email_verification: None,
            new_message: None,
            new_like: None,
            account_deletion_remainder_first: None,
            account_deletion_remainder_second: None,
            account_deletion_remainder_third: None,
            email_change_verification: None,
            email_change_notification: None,
            email_login: None,
            other: toml::Table::new(),
        }
    }
}

/// Merge the global email config with a per-message override, returning the effective
/// template, content type, and custom keys.
fn effective_email_config(
    global: &EmailConfig,
    resource: &Option<EmailContentStrings>,
) -> (String, bool, HashMap<String, StringResourceInternal>) {
    let msg = resource.as_ref().and_then(|r| r.email_config.as_ref());
    let template = msg
        .and_then(|c| c.template.clone())
        .unwrap_or_else(|| global.template.clone());
    let content_type_is_html = msg
        .and_then(|c| c.content_type_is_html)
        .unwrap_or(global.content_type_is_html);
    let custom_keys = msg
        .and_then(|c| c.custom_keys.clone())
        .unwrap_or_else(|| global.custom_keys.clone());
    (template, content_type_is_html, custom_keys)
}

/// Validate that all custom keys are referenced in the template.
fn validate_custom_keys(
    template: &str,
    custom_keys: &HashMap<String, StringResourceInternal>,
    context: &str,
) -> Result<(), ConfigFileError> {
    // Find all variable references in the template
    let mut referenced_keys = std::collections::HashSet::new();
    for line in template.lines() {
        for cap in line.match_indices("{") {
            if let Some(end_pos) = line[cap.0..].find("}") {
                let var_content = &line[cap.0 + 1..cap.0 + end_pos].trim();
                // Extract variable name
                let var_name = var_content.split_whitespace().next().unwrap_or("");
                if !var_name.is_empty() && var_name != "subject" && var_name != "body" {
                    referenced_keys.insert(var_name.to_string());
                }
            }
        }
    }

    // Check if all custom keys are referenced in the template
    for custom_key in custom_keys.keys() {
        if !referenced_keys.contains(custom_key) {
            return Err(ConfigFileError::InvalidConfig).attach(format!(
                "In email '{context}': custom key '{custom_key}' is defined but not referenced in the template",
            ));
        }
    }

    Ok(())
}

impl EmailContentFile {
    pub const CONFIG_FILE_NAME: &str = "email_content.toml";
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
            return Err(ConfigFileError::InvalidConfig).attach(format!(
                "Email content config file error. Unknown string resource '{key}'."
            ));
        }

        // Validate custom keys for the global config and each email's effective config
        let (template, _, custom_keys) = effective_email_config(&config.email, &None);
        validate_custom_keys(&template, &custom_keys, "global")?;

        let emails: [(&str, &Option<EmailContentStrings>); 9] = [
            ("email_verification", &config.email_verification),
            ("new_message", &config.new_message),
            ("new_like", &config.new_like),
            (
                "account_deletion_remainder_first",
                &config.account_deletion_remainder_first,
            ),
            (
                "account_deletion_remainder_second",
                &config.account_deletion_remainder_second,
            ),
            (
                "account_deletion_remainder_third",
                &config.account_deletion_remainder_third,
            ),
            (
                "email_change_verification",
                &config.email_change_verification,
            ),
            (
                "email_change_notification",
                &config.email_change_notification,
            ),
            ("email_login", &config.email_login),
        ];
        for (name, resource) in emails {
            let (template, _, custom_keys) = effective_email_config(&config.email, resource);
            validate_custom_keys(&template, &custom_keys, name)?;
        }

        if let Some(email_verification) = &config.email_verification
            && !email_verification.body.all_strings_contain("{token}")
        {
            return Err(ConfigFileError::InvalidConfig)
                .attach("'{token}' is missing from email_verification body text".to_string());
        }

        Ok(config)
    }

    pub fn email_body_content_type_is_html(&self) -> bool {
        self.email.content_type_is_html
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

        let rendered_body = render_template(
            &body,
            &body_data.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        );

        let (template, content_type_is_html, custom_keys) =
            effective_email_config(&self.config.email, resource);
        let mut data = vec![
            ("subject", subject.as_str()),
            ("body", rendered_body.as_str()),
        ];
        for (key, resource) in &custom_keys {
            let value = resource
                .translations
                .get(self.language)
                .unwrap_or(&resource.default);
            data.push((key.as_str(), value.as_str()));
        }

        let rendered = render_template(&template, &data);

        Ok(EmailContent {
            subject,
            body: rendered,
            body_is_html: content_type_is_html,
        })
    }

    pub fn email_verification(&self, token: &str) -> Result<EmailContent, ConfigFileError> {
        self.render_body_and_apply_template(
            &self.config.email_verification,
            "Verify your email address",
            "Please verify your email address by opening this link: https://example.com/verify_email?token={token}",
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

    pub fn email_change_verification(&self, token: &str) -> Result<EmailContent, ConfigFileError> {
        self.render_body_and_apply_template(
            &self.config.email_change_verification,
            "Verify your new email address",
            "Please verify your new email address by opening this link: https://example.com/verify_new_email?token={token}",
            HashMap::from_iter([("token", token)]),
        )
    }

    pub fn email_change_notification(&self) -> Result<EmailContent, ConfigFileError> {
        self.apply_template(
            &self.config.email_change_notification,
            "Email change notification",
            "Your account's email address will be changed. If you did not request this change, your account might have been compromised.",
        )
    }

    pub fn email_login(&self, token: &str) -> Result<EmailContent, ConfigFileError> {
        self.render_body_and_apply_template(
            &self.config.email_login,
            "Your login code",
            "Here is your login code: {token}",
            HashMap::from_iter([("token", token)]),
        )
    }
}
