//! Alert email subject and body formatting.

use manager_model::ManagerInstanceName;

use super::engine::AlertType;

/// Details of a breached threshold, used for email formatting.
#[derive(Debug, Clone)]
pub enum AlertDetails {
    /// Disk space usage percentage.
    DiskSpace(u32),
    /// RAM usage percentage.
    Ram(u32),
    /// Combined CPU usage percentage.
    Cpu(u32),
    /// Core index, core usage percentage and processes using the most CPU.
    CpuPerCore {
        core_index: usize,
        core_percent: u32,
        top_processes: Vec<(String, u32)>,
    },
    /// Received Mbit/s.
    BandwidthReceived(u32),
    /// Transmitted Mbit/s.
    BandwidthTransmitted(u32),
}

impl AlertDetails {
    pub fn description(&self) -> String {
        match self {
            AlertDetails::DiskSpace(percent) => {
                format!("Disk space usage is {percent}%")
            }
            AlertDetails::Ram(percent) => format!("RAM usage is {percent}%"),
            AlertDetails::Cpu(percent) => format!("Combined CPU usage is {percent}%"),
            AlertDetails::CpuPerCore {
                core_index,
                core_percent,
                top_processes,
            } => {
                let process_text = if top_processes.is_empty() {
                    String::new()
                } else {
                    let processes = top_processes
                        .iter()
                        .map(|(name, usage)| format!("{name} ({usage}%)"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(", processes using the most CPU: {processes}")
                };
                format!("CPU core {core_index} usage is {core_percent}%{process_text}")
            }
            AlertDetails::BandwidthReceived(mbits) => {
                format!("Bandwidth received usage is {mbits} Mbit/s")
            }
            AlertDetails::BandwidthTransmitted(mbits) => {
                format!("Bandwidth transmitted usage is {mbits} Mbit/s")
            }
        }
    }
}

pub fn alert_subject(manager_name: &ManagerInstanceName, alert_type: AlertType) -> String {
    format!("[ALERT] {manager_name}: {}", alert_type.name())
}

pub fn recovery_subject(manager_name: &ManagerInstanceName, alert_type: AlertType) -> String {
    format!("[RESOLVED] {manager_name}: {}", alert_type.name())
}

pub fn alert_body(alert_type: AlertType, details: &AlertDetails) -> String {
    format!(
        "{} has exceeded the alert threshold.\n\n{}\n\nA recovery email will be sent when the condition clears.",
        alert_type.name(),
        details.description()
    )
}

/// Sent after the last allowed email when the daily email
/// sending limit was reached with it.
pub fn alert_limit_reached_subject(
    manager_name: &ManagerInstanceName,
    alert_type: AlertType,
) -> String {
    format!("[LIMIT] {manager_name}: {}", alert_type.name())
}

/// Sent after the last allowed email when the daily email
/// sending limit was reached with it.
pub fn alert_limit_reached_body(alert_type: AlertType) -> String {
    format!(
        "Daily email sending limit reached for {} alert emails. Further emails are sent only after the daily limit resets.",
        alert_type.name()
    )
}

pub fn recovery_body(alert_type: AlertType) -> String {
    format!("{} is back below the alert threshold.", alert_type.name())
}
