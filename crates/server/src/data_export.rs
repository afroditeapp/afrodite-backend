use chrono::Utc;
use model::{DataExportName, DataExportState, DataExportType};
use server_api::app::DataExportManagerDataProvider;
use server_data::{
    app::ReadData,
    data_export::{DataExportCmd, DataExportReceiver},
};
use server_data_profile::read::GetReadProfileCommands;
use server_state::S;
use simple_backend::ServerQuitWatcher;
use tokio::task::JoinHandle;
use tracing::{error, warn};

#[derive(Debug)]
pub struct DataExportManagerQuitHandle {
    task: JoinHandle<()>,
}

impl DataExportManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("DataExportManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct DataExportManager {
    state: S,
}

impl DataExportManager {
    pub fn new_manager(
        receiver: DataExportReceiver,
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> DataExportManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(receiver, quit_notification));

        DataExportManagerQuitHandle { task }
    }

    pub async fn run(
        self,
        mut receiver: DataExportReceiver,
        mut quit_notification: ServerQuitWatcher,
    ) {
        loop {
            tokio::select! {
                item = receiver.0.recv() => {
                    match item {
                        Some(cmd) => {
                            self.handle_cmd(cmd).await;
                        }
                        None => {
                            error!("Data export event channel is broken");
                            return;
                        },
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    pub async fn handle_cmd(&self, cmd: DataExportCmd) {
        let zip_main_directory_name =
            match self.state.read().profile().profile(cmd.source().0).await {
                Ok(p) => {
                    let name = p
                        .profile
                        .name
                        .map(|v| v.into_string())
                        .unwrap_or_default()
                        .chars()
                        .map(|v| {
                            if v.is_ascii_alphanumeric() {
                                v.to_ascii_lowercase()
                            } else {
                                '$'
                            }
                        })
                        .collect::<String>();
                    let age = p.profile.age.value();
                    let time = Utc::now().format("%Y-%m-%d_%H-%M-%S");
                    let data_export_type = match cmd.data_export_type() {
                        DataExportType::User => "user",
                        DataExportType::Admin => "admin",
                    };
                    format!("data_export_{name}_{age}_{time}_{data_export_type}")
                }
                Err(e) => {
                    error!("Getting profile failed: {e:?}");
                    self.state
                        .data_export()
                        .update_state_if_export_ongoing(cmd.target(), DataExportState::error())
                        .await;
                    return;
                }
            };

        let next_state = match self
            .state
            .data_all_access()
            .data_export(zip_main_directory_name.clone(), cmd)
            .await
        {
            Err(e) => {
                error!("Data export failed: {e:?}");
                DataExportState::error()
            }
            Ok(()) => DataExportState::done(DataExportName {
                name: format!("{zip_main_directory_name}.zip"),
            }),
        };

        self.state
            .data_export()
            .update_state_if_export_ongoing(cmd.target(), next_state)
            .await;
    }
}
