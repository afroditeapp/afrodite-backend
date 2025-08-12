use std::{collections::HashMap, sync::Arc};

use model::{AccountIdInternal, DataExportState, DataExportStateType, DataExportType, UnixTime};
use server_common::result::{WrappedContextExt, WrappedResultExt};
use simple_backend_utils::time::DurationValue;
use tokio::sync::{
    Mutex, RwLock,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use crate::result::Result;

#[derive(thiserror::Error, Debug)]
pub enum DataExportError {
    #[error("Event sending failed")]
    EventSendingFailed,
    #[error("Export state not empty")]
    ExportStateNotEmpty,
    #[error("Export ongoing")]
    ExportOngoing,
}

/// Wrapper type for [AccountIdInternal], so that
/// is is not possible to use source as target
/// accidentally.
#[derive(Debug, Clone, Copy)]
pub struct SourceAccount(pub AccountIdInternal);

/// Wrapper type for [AccountIdInternal], so that
/// is is not possible to use target as source
/// accidentally.
#[derive(Debug, Clone, Copy)]
pub struct TargetAccount(pub AccountIdInternal);

#[derive(Debug, Clone, Copy)]
pub struct ExportCmd {
    source: SourceAccount,
    target: TargetAccount,
    data_export_type: DataExportType,
}

impl ExportCmd {
    pub fn source(&self) -> SourceAccount {
        self.source
    }

    pub fn target(&self) -> TargetAccount {
        self.target
    }

    pub fn data_export_type(&self) -> DataExportType {
        self.data_export_type
    }
}

#[derive(Debug)]
pub struct DataExportReceiver(pub UnboundedReceiver<ExportCmd>);

#[derive(Debug, Default, Clone)]
pub struct State {
    public_state: DataExportState,
    export_cmd_sent: UnixTime,
}

#[derive(Debug, Default, Clone)]
pub struct AccountSpecificData {
    state: Arc<Mutex<State>>,
}

impl AccountSpecificData {
    pub async fn get_public_state(&self) -> DataExportState {
        self.state.lock().await.public_state.clone()
    }

    pub async fn enough_time_elapsed_since_previous_export(&self) -> bool {
        self.state
            .lock()
            .await
            .export_cmd_sent
            .duration_value_elapsed(DurationValue::from_days(1))
    }
}

pub struct DataExportManagerData {
    event_queue: UnboundedSender<ExportCmd>,
    data: RwLock<HashMap<AccountIdInternal, AccountSpecificData>>,
}

impl DataExportManagerData {
    pub fn new() -> (Self, DataExportReceiver) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let notifier = DataExportReceiver(receiver);
        let data = Self {
            event_queue: sender,
            data: RwLock::new(HashMap::default()),
        };
        (data, notifier)
    }

    pub async fn send_export_cmd_if_export_file_is_not_in_use(
        &self,
        source: SourceAccount,
        target: TargetAccount,
        data_export_type: DataExportType,
    ) -> Result<(), DataExportError> {
        let target_state = self.get_state(target.0).await;
        let mut target_state = target_state.state.lock().await;
        if target_state.public_state.state != DataExportStateType::Empty {
            Err(DataExportError::ExportStateNotEmpty.report())
        } else {
            self.event_queue
                .send(ExportCmd {
                    source,
                    target,
                    data_export_type,
                })
                .change_context(DataExportError::EventSendingFailed)?;
            target_state.public_state = DataExportState::in_progress();
            target_state.export_cmd_sent = UnixTime::current_time();
            Ok(())
        }
    }

    pub async fn delete_state_if_export_not_ongoing(
        &self,
        target: AccountIdInternal,
    ) -> Result<(), DataExportError> {
        let target_state = self.get_state(target).await;
        let mut target_state = target_state.state.lock().await;
        if target_state.public_state.state == DataExportStateType::InProgress {
            Err(DataExportError::ExportOngoing.report())
        } else {
            target_state.public_state = DataExportState::empty();
            Ok(())
        }
    }

    pub async fn update_state_if_export_ongoing(
        &self,
        target: TargetAccount,
        state: DataExportState,
    ) {
        let target_state = self.get_state(target.0).await;
        let mut target_state = target_state.state.lock().await;
        if target_state.public_state.state == DataExportStateType::InProgress {
            target_state.public_state = state;
        }
    }

    pub async fn get_state(&self, account_id: AccountIdInternal) -> AccountSpecificData {
        let state = self.data.read().await.get(&account_id).cloned();

        if let Some(state) = state {
            return state;
        }

        let mut w = self.data.write().await;
        let state = w.entry(account_id).or_default();
        state.clone()
    }
}
