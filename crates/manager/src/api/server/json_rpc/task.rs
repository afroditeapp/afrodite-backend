

use manager_model::JsonRpcResponse;
use tracing::warn;
use crate::{api::{GetConfig, GetTaskManager}, server::task::TaskManagerMessage};

use error_stack::{Result, ResultExt};

use super::JsonRpcError;

pub trait RpcTask: GetConfig + GetTaskManager {
    async fn rpc_trigger_backend_data_reset(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        if self.config().manual_tasks_config().allow_backend_data_reset.is_none() {
            warn!("Skipping backend data reset request because it is disabled from config file");
            return Ok(JsonRpcResponse::successful())
        }

        self.task_manager()
            .send_message(TaskManagerMessage::BackendDataReset)
            .await
            .change_context(JsonRpcError::TaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_trigger_backend_restart(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        if !self.config().manual_tasks_config().allow_backend_restart {
            warn!("Skipping backend restart request because it is disabled from config file");
            return Ok(JsonRpcResponse::successful())
        }

        self.task_manager()
            .send_message(TaskManagerMessage::BackendRestart)
            .await
            .change_context(JsonRpcError::TaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_trigger_system_reboot(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        if !self.config().manual_tasks_config().allow_system_reboot {
            warn!("Skipping system reboot request because it is disabled from config file");
            return Ok(JsonRpcResponse::successful())
        }

        self.task_manager()
        .send_message(TaskManagerMessage::SystemReboot)
            .await
            .change_context(JsonRpcError::TaskManager)?;
        Ok(JsonRpcResponse::successful())
    }
}

impl <T: GetConfig + GetTaskManager> RpcTask for T {}
