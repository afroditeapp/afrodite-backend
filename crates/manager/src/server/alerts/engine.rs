//! Alert state machine.
//!
//! Tracks sustained threshold breaches per alert type and decides when
//! alert and recovery emails should be sent. Email sending limits are
//! handled separately in `email_limit`.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertType {
    DiskSpace,
    Ram,
    Cpu,
    CpuPerCore,
    BandwidthReceived,
    BandwidthTransmitted,
}

impl AlertType {
    pub fn name(self) -> &'static str {
        match self {
            AlertType::DiskSpace => "Disk space",
            AlertType::Ram => "RAM",
            AlertType::Cpu => "CPU",
            AlertType::CpuPerCore => "CPU per core",
            AlertType::BandwidthReceived => "Bandwidth received",
            AlertType::BandwidthTransmitted => "Bandwidth transmitted",
        }
    }

    /// All alert types.
    pub fn all() -> [AlertType; 6] {
        [
            AlertType::DiskSpace,
            AlertType::Ram,
            AlertType::Cpu,
            AlertType::CpuPerCore,
            AlertType::BandwidthReceived,
            AlertType::BandwidthTransmitted,
        ]
    }
}

/// Current state of one alert type.
#[derive(Debug, Default)]
struct AlertState {
    consecutive_breaches: u32,
    triggered: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertDecision {
    /// No email should be sent.
    None,
    /// Condition became active, send an alert email.
    Alert,
    /// Condition cleared, send a recovery email.
    Recovery,
}

pub struct AlertEngine {
    states: HashMap<AlertType, AlertState>,
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self {
            states: AlertType::all()
                .into_iter()
                .map(|alert_type| (alert_type, AlertState::default()))
                .collect(),
        }
    }
}

impl AlertEngine {
    /// Process one check result for an alert type.
    ///
    /// * `breached` - is the threshold currently exceeded
    /// * `sustained_checks` - how many consecutive breaches are required
    ///   before the first alert
    pub fn process(
        &mut self,
        alert_type: AlertType,
        breached: bool,
        sustained_checks: u32,
    ) -> AlertDecision {
        let state = self
            .states
            .get_mut(&alert_type)
            .expect("Alert state should exist for all alert types");

        if !breached {
            state.consecutive_breaches = 0;
            return if state.triggered {
                state.triggered = false;
                AlertDecision::Recovery
            } else {
                AlertDecision::None
            };
        }

        state.consecutive_breaches += 1;

        if state.triggered {
            // Condition is already active, no new email.
            AlertDecision::None
        } else if state.consecutive_breaches >= sustained_checks {
            state.triggered = true;
            AlertDecision::Alert
        } else {
            AlertDecision::None
        }
    }
}
