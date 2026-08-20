use regex::Regex;

/// What is wrong with an email address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailAddressValidationError {
    Empty,
    ContainsWhitespace,
    MultipleAtSigns,
    InvalidCharacters,
    MissingAlphanumeric,
    ConsecutiveDots,
    LocalPartLeadingOrTrailingDot,
    DomainLeadingOrTrailingDot,
    DomainMissingDot,
}

/// Validates email addresses during registration.
///
/// The regexes are compiled once when the validator is created so that
/// validation does not pay the compilation cost on every call.
pub struct EmailAddressValidator {
    whitespace: Regex,
    allowed_local_part: Regex,
    alphanumeric: Regex,
}

impl EmailAddressValidator {
    pub fn new() -> Self {
        Self {
            whitespace: Regex::new(r"\s").unwrap(),
            allowed_local_part: Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap(),
            alphanumeric: Regex::new(r"[a-zA-Z0-9]").unwrap(),
        }
    }

    /// Returns the reason why `email` is invalid, or `None` if it is valid.
    pub fn validate(&self, email: &str) -> Option<EmailAddressValidationError> {
        let trimmed = email.trim();
        if trimmed.is_empty() {
            return Some(EmailAddressValidationError::Empty);
        }
        // No whitespace allowed anywhere in the address.
        if self.whitespace.is_match(trimmed) {
            return Some(EmailAddressValidationError::ContainsWhitespace);
        }

        // Only a single `@` is allowed.
        let mut parts = trimmed.split('@');
        let local_part = parts.next().unwrap();
        let domain_part = match parts.next() {
            Some(domain) => domain,
            None => return Some(EmailAddressValidationError::MultipleAtSigns),
        };
        if parts.next().is_some() {
            return Some(EmailAddressValidationError::MultipleAtSigns);
        }

        // Every character of the local part must be allowed. `+` is excluded as
        // the backend does not support address aliases.
        if !self.allowed_local_part.is_match(local_part) {
            return Some(EmailAddressValidationError::InvalidCharacters);
        }

        // The local part must contain at least one alphanumeric character.
        if !self.alphanumeric.is_match(local_part) {
            return Some(EmailAddressValidationError::MissingAlphanumeric);
        }

        // Subsequent dots are not allowed.
        if local_part.contains("..") || domain_part.contains("..") {
            return Some(EmailAddressValidationError::ConsecutiveDots);
        }

        // Local part must not start or end with a dot.
        if local_part.starts_with('.') || local_part.ends_with('.') {
            return Some(EmailAddressValidationError::LocalPartLeadingOrTrailingDot);
        }

        // Domain part must not start or end with a dot and must contain a dot (TLD).
        if domain_part.starts_with('.') || domain_part.ends_with('.') {
            return Some(EmailAddressValidationError::DomainLeadingOrTrailingDot);
        }
        if !domain_part.contains('.') {
            return Some(EmailAddressValidationError::DomainMissingDot);
        }

        None
    }
}

impl Default for EmailAddressValidator {
    fn default() -> Self {
        Self::new()
    }
}
