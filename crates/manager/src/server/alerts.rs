//! Periodic system health monitoring with email alerts.

use std::time::Duration;

use manager_config::file::AlertsConfig;
use manager_model::ManagerInstanceName;
use simple_backend_model::UnixTime;
use tokio::{task::JoinHandle, time::Instant};
use tracing::{error, info, warn};

use super::ServerQuitWatcher;

pub mod email;
pub mod email_limit;
pub mod engine;
pub mod message;
pub mod metrics;

use email_limit::{EmailLimit, EmailSendingPermission};
use engine::{AlertDecision, AlertEngine, AlertType};
use message::AlertDetails;
use metrics::MetricsCollector;

#[derive(Debug)]
pub struct AlertManagerQuitHandle {
    task: JoinHandle<()>,
}

impl AlertManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("AlertManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct AlertManager {
    config: AlertsConfig,
    manager_name: ManagerInstanceName,
    sender: email::AlertEmailSender,
    collector: MetricsCollector,
    engine: AlertEngine,
    email_limit: EmailLimit,
}

impl AlertManager {
    pub fn new_manager(
        config: &AlertsConfig,
        manager_name: ManagerInstanceName,
        quit_notification: ServerQuitWatcher,
    ) -> simple_backend_utils::Result<AlertManagerQuitHandle, email::AlertEmailError> {
        let sender = email::AlertEmailSender::new(&config.email)?;

        let manager = Self {
            config: config.clone(),
            manager_name,
            sender,
            collector: MetricsCollector::default(),
            engine: AlertEngine::default(),
            email_limit: EmailLimit::new(config.max_emails_per_alert_per_day()),
        };

        let task = tokio::spawn(manager.run(quit_notification));

        Ok(AlertManagerQuitHandle { task })
    }

    async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        let interval_seconds = self.config.check_interval().seconds as u64;
        let first_tick = Instant::now() + Duration::from_secs(interval_seconds);
        let mut timer = tokio::time::interval_at(first_tick, Duration::from_secs(interval_seconds));

        info!("Alert manager started");

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    self.run_checks().await;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    /// Send a separate email about the reached daily email sending limit.
    /// Called after the email which consumed the last allowed slot.
    async fn send_limit_reached_email(
        &self,
        alert_type: AlertType,
    ) -> simple_backend_utils::Result<(), email::AlertEmailError> {
        let subject = message::alert_limit_reached_subject(&self.manager_name, alert_type);
        let body = message::alert_limit_reached_body(alert_type);
        self.sender
            .send(&self.config.email.recipient, &subject, &body)
            .await
    }

    async fn run_checks(&mut self) {
        let metrics = self.collector.collect().await;

        if self.config.debug_logging() {
            info!("Collected system metrics: {:#?}", metrics);
        }

        let thresholds = self.config.thresholds();
        let sustained = self.config.sustained_checks();

        let received_mbits = Self::bytes_per_second_to_mbits(metrics.received_bytes_per_second);
        let transmitted_mbits =
            Self::bytes_per_second_to_mbits(metrics.transmitted_bytes_per_second);

        let checks = [
            (
                AlertType::DiskSpace,
                metrics.disk_space_percent >= thresholds.disk_space_percent,
                sustained.disk_space,
                AlertDetails::DiskSpace(metrics.disk_space_percent),
            ),
            (
                AlertType::Ram,
                metrics.ram_percent >= thresholds.ram_percent,
                sustained.ram,
                AlertDetails::Ram(metrics.ram_percent),
            ),
            (
                AlertType::Cpu,
                metrics.cpu_percent >= thresholds.cpu_percent,
                sustained.cpu,
                AlertDetails::Cpu(metrics.cpu_percent),
            ),
            (
                AlertType::CpuPerCore,
                metrics.cpu_per_core_percent >= thresholds.cpu_per_core_percent,
                sustained.cpu_per_core,
                AlertDetails::CpuPerCore {
                    core_index: metrics.cpu_per_core_index,
                    core_percent: metrics.cpu_per_core_percent,
                    top_processes: metrics.top_processes.clone(),
                },
            ),
            (
                AlertType::BandwidthReceived,
                received_mbits >= thresholds.bandwidth_received_mbits,
                sustained.bandwidth_received,
                AlertDetails::BandwidthReceived(received_mbits),
            ),
            (
                AlertType::BandwidthTransmitted,
                transmitted_mbits >= thresholds.bandwidth_transmitted_mbits,
                sustained.bandwidth_transmitted,
                AlertDetails::BandwidthTransmitted(transmitted_mbits),
            ),
        ];

        for (alert_type, breached, sustained_checks, details) in checks {
            let decision = self.engine.process(alert_type, breached, sustained_checks);

            let result = match decision {
                AlertDecision::None => Ok(()),
                AlertDecision::Alert => {
                    let permission = self
                        .email_limit
                        .email_sending_allowed(alert_type, UnixTime::current_time());

                    if permission == EmailSendingPermission::LimitReached {
                        warn!(
                            "Daily email sending limit reached, {} alert email not sent",
                            alert_type.name()
                        );
                        Ok(())
                    } else {
                        let subject = message::alert_subject(&self.manager_name, alert_type);
                        let body = message::alert_body(alert_type, &details);
                        let result = self
                            .sender
                            .send(&self.config.email.recipient, &subject, &body)
                            .await;

                        if result.is_ok() && permission == EmailSendingPermission::AllowedLast {
                            self.send_limit_reached_email(alert_type).await
                        } else {
                            result
                        }
                    }
                }
                AlertDecision::Recovery => {
                    let permission = self
                        .email_limit
                        .email_sending_allowed(alert_type, UnixTime::current_time());

                    if permission == EmailSendingPermission::LimitReached {
                        warn!(
                            "Daily email sending limit reached, {} recovery email not sent",
                            alert_type.name()
                        );
                        Ok(())
                    } else {
                        let subject = message::recovery_subject(&self.manager_name, alert_type);
                        let body = message::recovery_body(alert_type);
                        let result = self
                            .sender
                            .send(&self.config.email.recipient, &subject, &body)
                            .await;

                        if result.is_ok() && permission == EmailSendingPermission::AllowedLast {
                            self.send_limit_reached_email(alert_type).await
                        } else {
                            result
                        }
                    }
                }
            };

            if let Err(e) = result {
                error!(
                    "Sending {} alert email failed. Error: {:?}",
                    alert_type.name(),
                    e
                );
            }
        }
    }

    fn bytes_per_second_to_mbits(bytes_per_second: u64) -> u32 {
        ((bytes_per_second * 8) / 1000 / 1000) as u32
    }
}
