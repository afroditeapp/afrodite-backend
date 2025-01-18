use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;

use crate::{SecureStorageEncryptionKey, SoftwareUpdateStatus, SystemInfo, SystemInfoList};

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
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcRequest {
    /// If instance name is not found, then
    /// [JsonRpcResponseType::RequestReceiverNotFound]
    pub receiver: ManagerInstanceName,
    pub request: JsonRpcRequestType,
}

impl JsonRpcRequest {
    pub fn get_secure_storage_encryption_key(
        receiver: ManagerInstanceName,
        key: ManagerInstanceName,
    ) -> Self {
        Self {
            receiver,
            request: JsonRpcRequestType::GetSecureStorageEncryptionKey(key),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum JsonRpcRequestType {
    /// Response [JsonRpcResponseType::ManagerInstanceNames]
    GetManagerInstanceNames,
    /// Response [JsonRpcResponseType::SecureStorageEncryptionKey]
    GetSecureStorageEncryptionKey(ManagerInstanceName),
    /// Response [JsonRpcResponseType::SystemInfoList]
    GetSystemInfo,
    /// Response [JsonRpcResponseType::SoftwareUpdateStatus]
    GetSoftwareUpdateStatus,
    /// Response [JsonRpcResponseType::Successful]
    TriggerSoftwareUpdateDownload,
    /// Response [JsonRpcResponseType::Successful]
    TriggerSoftwareUpdateInstall,
    /// Response [JsonRpcResponseType::Successful]
    TriggerSystemReboot,
    /// Response [JsonRpcResponseType::Successful]
    TriggerBackendDataReset,
    /// Response [JsonRpcResponseType::Successful]
    ScheduleReboot,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcResponse {
    response: JsonRpcResponseType,
}

impl JsonRpcResponse {
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
            response: JsonRpcResponseType::SystemInfoList(
                SystemInfoList {
                    info: vec![info],
                }
            )
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

    pub fn into_response(self) -> JsonRpcResponseType {
        self.response
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ServerEvent {
    event: ServerEventType,
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
    SystemInfoList(SystemInfoList),
    SoftwareUpdateStatus(SoftwareUpdateStatus),
    Successful,
    RequestReceiverNotFound,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ManagerInstanceNameList {
    pub names: Vec<ManagerInstanceName>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ManagerInstanceName(pub String);

impl std::fmt::Display for ManagerInstanceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ServerEventType {
    RebootScheduled(RebootTime),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RebootTime(pub UnixTime);
