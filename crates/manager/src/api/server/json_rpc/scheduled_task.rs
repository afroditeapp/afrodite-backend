use error_stack::{Result, ResultExt};
use manager_model::{JsonRpcResponse, ManagerApiScheduledTaskType};

use super::JsonRpcError;
use crate::{
    api::{GetConfig, GetScheduledTaskManager},
    server::scheduled_task::ScheduledTaskManagerMessage,
};

pub trait RpcScheduledTask: GetConfig + GetScheduledTaskManager {
    async fn rpc_schedule_task(
        &self,
        task: ManagerApiScheduledTaskType,
        notify_backend: bool,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.scheduled_task_manager()
            .send_message(ScheduledTaskManagerMessage::Schedule {
                task,
                notify_backend,
            })
            .await
            .change_context(JsonRpcError::ScheduledTaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_unschedule_task(
        &self,
        task: ManagerApiScheduledTaskType,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.scheduled_task_manager()
            .send_message(ScheduledTaskManagerMessage::Unschedule { task })
            .await
            .change_context(JsonRpcError::ScheduledTaskManager)?;
        Ok(JsonRpcResponse::successful())
    }

    async fn rpc_get_scheduled_tasks_status(&self) -> Result<JsonRpcResponse, JsonRpcError> {
        let status = self.scheduled_task_manager().status().await;
        Ok(JsonRpcResponse::scheduled_tasks_status(status))
    }
}

impl<T: GetConfig + GetScheduledTaskManager> RpcScheduledTask for T {}
