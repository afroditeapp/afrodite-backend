use std::{collections::HashMap, io::Write, path::Path};

use error_stack::{Result, ResultExt};
use handlebars::Handlebars;
use model::StringResourceInternal;
use serde::Deserialize;
use serde_json::json;
use toml::map::Map;

use crate::file::ConfigFileError;

const DEFAULT_WEB_CONTENT: &str = r#"
# Web page template (non-translatable, required)
# Available variables: title, body
web_page_template = """
{{title}}

{{body}}
"""

web_page_content_type_is_html = false

# Access Denied Page

[access_denied.title]
default = "Access Denied"

[access_denied.body]
default = "Sorry, access to this application is not allowed from your current IP address.\n\nYour IP: {{ip_address}}\n\nIf you believe this is an error, please contact the system administrator."

# Email Confirmation Page

[email_confirmed.title]
default = "Email Confirmed"

[email_confirmed.body]
default = "Email confirmed successfully!"

[email_confirmation_invalid.title]
default = "Invalid Token"

[email_confirmation_invalid.body]
default = "Invalid or expired token"

"#;

#[derive(Debug, Clone)]
pub struct WebContent {
    pub content: String,
    pub is_html: bool,
}

#[derive(Debug, Default, Deserialize)]
struct WebContentStrings {
    title: StringResourceInternal,
    body: StringResourceInternal,
}

#[derive(Debug, Deserialize)]
pub struct WebContentFile {
    web_page_template: String,
    web_page_content_type_is_html: bool,
    access_denied: Option<WebContentStrings>,
    email_confirmed: Option<WebContentStrings>,
    email_confirmation_invalid: Option<WebContentStrings>,
    #[serde(flatten)]
    other: toml::Table,
}

const DEFAULT_TEMPLATE: &str = "
{{title}}

{{body}}
";

impl Default for WebContentFile {
    fn default() -> Self {
        Self {
            web_page_template: DEFAULT_TEMPLATE.to_string(),
            web_page_content_type_is_html: false,
            access_denied: None,
            email_confirmed: None,
            email_confirmation_invalid: None,
            other: Map::new(),
        }
    }
}

impl WebContentFile {
    pub fn load(
        file: impl AsRef<Path>,
        save_if_needed: bool,
    ) -> Result<WebContentFile, ConfigFileError> {
        let path = file.as_ref();
        if !path.exists() && save_if_needed {
            let mut new_file =
                std::fs::File::create_new(path).change_context(ConfigFileError::LoadConfig)?;
            new_file
                .write_all(DEFAULT_WEB_CONTENT.as_bytes())
                .change_context(ConfigFileError::LoadConfig)?;
        }
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: WebContentFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        if let Some(key) = config.other.keys().next() {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "Page content config file error. Unknown string resource '{key}'."
            ));
        }

        // Validate that template can be parsed
        if let Err(e) = Handlebars::new().render_template_with_context_to_write(
            &config.web_page_template,
            &handlebars::Context::null(),
            &mut std::io::sink(),
        ) {
            return Err(ConfigFileError::InvalidConfig)
                .attach_printable(format!("Template parsing error: {e}"));
        }

        Ok(config)
    }

    pub fn get<'a, T: AsRef<str>>(&'a self, language: Option<&'a T>) -> WebStringGetter<'a> {
        WebStringGetter {
            config: self,
            language: language.map(|v| v.as_ref()).unwrap_or_default(),
        }
    }
}

pub struct WebStringGetter<'a> {
    config: &'a WebContentFile,
    language: &'a str,
}

impl<'a> WebStringGetter<'a> {
    fn render_web_page(
        &self,
        resource: &Option<WebContentStrings>,
        default_title: &str,
        default_body: &str,
    ) -> Result<WebContent, ConfigFileError> {
        self.render_body_and_web_page(resource, default_title, default_body, HashMap::new())
    }

    fn render_body_and_web_page(
        &self,
        resource: &Option<WebContentStrings>,
        default_title: &str,
        default_body: &str,
        body_data: HashMap<&str, &str>,
    ) -> Result<WebContent, ConfigFileError> {
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

        let rendered_body = Handlebars::new()
            .render_template(&body, &body_data)
            .change_context(ConfigFileError::InvalidConfig)
            .attach_printable_lazy(|| "Body template rendering error".to_string())?;

        let data = json!({
            "title": title,
            "body": rendered_body,
        });

        let rendered = Handlebars::new()
            .render_template(&self.config.web_page_template, &data)
            .change_context(ConfigFileError::InvalidConfig)
            .attach_printable_lazy(|| "Template rendering error".to_string())?;

        Ok(WebContent {
            content: rendered,
            is_html: self.config.web_page_content_type_is_html,
        })
    }

    pub fn access_denied(&self, ip_address: &str) -> Result<WebContent, ConfigFileError> {
        self.render_body_and_web_page(
            &self.config.access_denied,
            "Access Denied",
            "Sorry, access to this application is not allowed from your current IP address.\n\nYour IP: {{ip_address}}\n\nIf you believe this is an error, please contact the system administrator.",
            HashMap::from_iter([("ip_address", ip_address)]),
        )
    }

    pub fn email_confirmed(&self) -> Result<WebContent, ConfigFileError> {
        self.render_web_page(
            &self.config.email_confirmed,
            "Email Confirmed",
            "Email confirmed successfully!",
        )
    }

    pub fn email_confirmation_invalid(&self) -> Result<WebContent, ConfigFileError> {
        self.render_web_page(
            &self.config.email_confirmation_invalid,
            "Invalid Token",
            "Invalid or expired token",
        )
    }
}
