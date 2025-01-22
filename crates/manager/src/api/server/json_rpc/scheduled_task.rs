

use manager_model::JsonRpcResponse;
use crate::{api::{GetConfig, GetScheduledTaskManager}, server::scheduled_task::ScheduledTaskManagerMessage};

use error_stack::{Result, ResultExt};

use super::JsonRpcError;

pub trait RpcScheduledTask: GetConfig + GetScheduledTaskManager {
    async fn rpc_schedule_backend_restart(
        &self,
        notify_backend: bool,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.scheduled_task_manager()
            .send_message(ScheduledTaskManagerMessage::ScheduleBackendRestart { notify_backend })
            .await
            .change_context(JsonRpcError::ScheduledTaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_schedule_system_reboot(
        &self,
        notify_backend: bool,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.scheduled_task_manager()
            .send_message(ScheduledTaskManagerMessage::ScheduleSystemReboot { notify_backend })
            .await
            .change_context(JsonRpcError::ScheduledTaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_unschedule_backend_restart(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.scheduled_task_manager()
            .send_message(ScheduledTaskManagerMessage::UnscheduleBackendRestart)
            .await
            .change_context(JsonRpcError::ScheduledTaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_unschedule_system_reboot(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.scheduled_task_manager()
            .send_message(ScheduledTaskManagerMessage::UnscheduleSystemReboot)
            .await
            .change_context(JsonRpcError::ScheduledTaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_get_scheduled_tasks_status(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        let status  = self.scheduled_task_manager()
            .status()
            .await;
        Ok(JsonRpcResponse::scheduled_tasks_status(status))
    }
}

impl <T: GetConfig + GetScheduledTaskManager> RpcScheduledTask for T {}
