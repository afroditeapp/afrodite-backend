//! Daily email sending limits per alert type.

use std::collections::HashMap;

use simple_backend_model::UnixTime;

use super::engine::AlertType;

/// Whether an email can be sent for an alert type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailSendingPermission {
    /// Email can be sent.
    Allowed,
    /// Email can be sent and it is the last email allowed during
    /// the current UTC day.
    AllowedLast,
    /// Daily email limit has been reached, email cannot be sent.
    LimitReached,
}

#[derive(Debug)]
struct EmailCount {
    /// UTC day of the counter.
    day: Option<jiff::civil::Date>,
    /// Emails sent during the current UTC day.
    sent: u32,
}

/// Tracks sent emails per alert type and enforces the daily email
/// sending limit. Both alert and recovery emails are counted.
#[derive(Debug)]
pub struct EmailLimit {
    max_emails_per_day: u32,
    counters: HashMap<AlertType, EmailCount>,
}

impl EmailLimit {
    pub fn new(max_emails_per_day: u32) -> Self {
        Self {
            max_emails_per_day,
            counters: AlertType::all()
                .into_iter()
                .map(|alert_type| (alert_type, EmailCount { day: None, sent: 0 }))
                .collect(),
        }
    }

    /// Check if an email can be sent for the alert type and reserve one
    /// slot from the daily counter. Should be called right after
    /// `AlertEngine::process` when the decision requires sending an email.
    pub fn email_sending_allowed(
        &mut self,
        alert_type: AlertType,
        current_time: UnixTime,
    ) -> EmailSendingPermission {
        let count = self
            .counters
            .get_mut(&alert_type)
            .expect("Email count should exist for all alert types");

        let current_day = current_time
            .to_jiff_time()
            .map(|time| time.to_zoned(jiff::tz::TimeZone::UTC).date());
        if count.day != current_day {
            count.day = current_day;
            count.sent = 0;
        }

        if count.sent >= self.max_emails_per_day {
            return EmailSendingPermission::LimitReached;
        }

        count.sent += 1;

        if count.sent >= self.max_emails_per_day {
            EmailSendingPermission::AllowedLast
        } else {
            EmailSendingPermission::Allowed
        }
    }
}
