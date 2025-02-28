use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

use crate::{ManualTaskType, NotifyBackend, ScheduledTaskStatus, ScheduledTaskType, SecureStorageEncryptionKey, SoftwareUpdateStatus, SoftwareUpdateTaskType, SystemInfo};

#[derive(Debug, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum ManagerProtocolVersion {
    V1 = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum ManagerProtocolMode {
    JsonRpc = 0,
    ListenServerEvents = 1,
    JsonRpcLink = 2,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcRequest {
    /// If instance name is not found, then
    /// [JsonRpcResponseType::RequestReceiverNotFound]
    pub receiver: ManagerInstanceName,
    pub request: JsonRpcRequestType,
}

impl JsonRpcRequest {
    pub fn new(receiver: ManagerInstanceName, request: JsonRpcRequestType) -> Self {
        Self {
            receiver,
            request,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum JsonRpcRequestType {
    /// Response [JsonRpcResponseType::ManagerInstanceNames]
    GetManagerInstanceNames,
    /// Response [JsonRpcResponseType::SecureStorageEncryptionKey]
    GetSecureStorageEncryptionKey(ManagerInstanceName),
    /// Response [JsonRpcResponseType::SystemInfo]
    GetSystemInfo,
    /// Response [JsonRpcResponseType::SoftwareUpdateStatus]
    GetSoftwareUpdateStatus,
    /// Response [JsonRpcResponseType::Successful]
    TriggerSoftwareUpdateTask(SoftwareUpdateTaskType),
    /// Response [JsonRpcResponseType::Successful]
    TriggerManualTask(ManualTaskType),
    /// Response [JsonRpcResponseType::ScheduledTasksStatus]
    GetScheduledTasksStatus,
    /// Response [JsonRpcResponseType::Successful]
    ScheduleTask(ScheduledTaskType, NotifyBackend),
    /// Response [JsonRpcResponseType::Successful]
    UnscheduleTask(ScheduledTaskType),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcResponse {
    response: JsonRpcResponseType,
}

impl JsonRpcResponse {
    pub fn successful() -> Self {
        Self {
            response: JsonRpcResponseType::Successful,
        }
    }

    pub fn request_receiver_not_found() -> Self {
        Self {
            response: JsonRpcResponseType::RequestReceiverNotFound,
        }
    }

    pub fn secure_storage_encryption_key(
        key: SecureStorageEncryptionKey,
    ) -> Self {
        Self {
            response: JsonRpcResponseType::SecureStorageEncryptionKey(key),
        }
    }

    pub fn manager_instance_names(
        names: Vec<ManagerInstanceName>,
    ) -> Self {
        Self {
            response: JsonRpcResponseType::ManagerInstanceNames(
                ManagerInstanceNameList { names }
            ),
        }
    }

    pub fn system_info(
        info: SystemInfo,
    ) -> Self {
        Self {
            response: JsonRpcResponseType::SystemInfo(info)
        }
    }

    pub fn software_update_status(
        status: SoftwareUpdateStatus,
    ) -> Self {
        Self {
            response: JsonRpcResponseType::SoftwareUpdateStatus(
                status
            ),
        }
    }

    pub fn scheduled_tasks_status(
        status: ScheduledTaskStatus,
    ) -> Self {
        Self {
            response: JsonRpcResponseType::ScheduledTasksStatus(
                status
            ),
        }
    }

    pub fn into_response(self) -> JsonRpcResponseType {
        self.response
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ServerEvent {
    pub event: ServerEventType,
}

impl ServerEvent {
    pub fn event(&self) -> &ServerEventType {
        &self.event
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum JsonRpcResponseType {
    ManagerInstanceNames(ManagerInstanceNameList),
    SecureStorageEncryptionKey(SecureStorageEncryptionKey),
    SystemInfo(SystemInfo),
    SoftwareUpdateStatus(SoftwareUpdateStatus),
    ScheduledTasksStatus(ScheduledTaskStatus),
    Successful,
    RequestReceiverNotFound,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ManagerInstanceNameList {
    pub names: Vec<ManagerInstanceName>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, ToSchema)]
pub struct ManagerInstanceName(pub String);

impl ManagerInstanceName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl std::fmt::Display for ManagerInstanceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema, IntoParams)]
pub struct ManagerInstanceNameValue {
    pub manager_name: ManagerInstanceName,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ServerEventType {
    MaintenanceSchedulingStatus(Option<MaintenanceTime>),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct MaintenanceTime(pub UnixTime);
