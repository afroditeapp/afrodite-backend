use error_stack::{Result, ResultExt};
use manager_model::{JsonRpcResponse, ManualTaskType};
use tracing::warn;

use super::JsonRpcError;
use crate::api::{GetConfig, GetTaskManager};

pub trait RpcTask: GetConfig + GetTaskManager {
    async fn rpc_trigger_manual_task(
        &self,
        task: ManualTaskType,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        match task {
            ManualTaskType::BackendDataReset => {
                if self
                    .config()
                    .manual_tasks_config()
                    .allow_backend_data_reset
                    .is_none()
                {
                    warn!(
                        "Skipping backend data reset request because it is disabled from config file"
                    );
                    return Ok(JsonRpcResponse::successful());
                }
            }
            ManualTaskType::BackendRestart => {
                if !self.config().manual_tasks_config().allow_backend_restart {
                    warn!(
                        "Skipping backend restart request because it is disabled from config file"
                    );
                    return Ok(JsonRpcResponse::successful());
                }
            }
            ManualTaskType::SystemReboot => {
                if !self.config().manual_tasks_config().allow_system_reboot {
                    warn!("Skipping system reboot request because it is disabled from config file");
                    return Ok(JsonRpcResponse::successful());
                }
            }
        }

        self.task_manager()
            .send_message(task)
            .await
            .change_context(JsonRpcError::TaskManager)?;
        Ok(JsonRpcResponse::successful())
    }
}

impl<T: GetConfig + GetTaskManager> RpcTask for T {}
