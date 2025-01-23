use error_stack::{report, Result, ResultExt};
use manager_model::{ManualTaskType, NotifyBackend, ScheduledTaskStatus, ScheduledTaskType, SoftwareUpdateTaskType};
use manager_model::{JsonRpcRequest, JsonRpcRequestType, JsonRpcResponse, JsonRpcResponseType, ManagerInstanceName, ManagerInstanceNameList, ManagerProtocolMode, ManagerProtocolVersion, SecureStorageEncryptionKey, ServerEvent, SoftwareUpdateStatus, SystemInfo};

use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

use crate::{ClientError, ManagerClient};



pub trait ClientConnectionReadWrite: ClientConnectionRead + ClientConnectionWrite {}
impl <T: ClientConnectionRead + ClientConnectionWrite> ClientConnectionReadWrite for T {}

pub trait ClientConnectionRead: tokio::io::AsyncRead + Send + std::marker::Unpin + 'static {}
impl <T: tokio::io::AsyncRead + Send + std::marker::Unpin + 'static> ClientConnectionRead for T {}

pub trait ClientConnectionWrite: tokio::io::AsyncWrite + Send + std::marker::Unpin + 'static {}
impl <T: tokio::io::AsyncWrite + Send + std::marker::Unpin + 'static> ClientConnectionWrite for T {}

pub trait ConnectionUtilsRead: tokio::io::AsyncRead + Unpin  {
    async fn receive_u8(&mut self) -> Result<u8, ClientError> {
        self.read_u8().await.change_context(ClientError::Read)
    }

    async fn receive_string_with_u32_len(&mut self) -> Result<String, ClientError> {
        let len = self.read_u32_le().await.change_context(ClientError::Read)?;
        let len_usize: usize = TryInto::<usize>::try_into(len).change_context(ClientError::UnsupportedStringLength)?;
        let mut vec: Vec<u8> = vec![0; len_usize];
        self.read_exact(&mut vec).await.change_context(ClientError::Read)?;
        String::from_utf8(vec).change_context(ClientError::Parse)
    }

    async fn receive_protocol_version(&mut self) -> Result<ManagerProtocolVersion, ClientError> {
        TryInto::<ManagerProtocolVersion>::try_into(self.receive_u8().await?)
            .change_context(ClientError::Parse)
    }

    async fn receive_protocol_mode(&mut self) -> Result<ManagerProtocolMode, ClientError> {
        TryInto::<ManagerProtocolMode>::try_into(self.receive_u8().await?)
            .change_context(ClientError::Parse)
    }

    async fn receive_json_rpc_request(&mut self) -> Result<JsonRpcRequest, ClientError> {
        let s = self.receive_string_with_u32_len().await
            .change_context(ClientError::Read)?;
        serde_json::from_str(&s)
            .change_context(ClientError::Parse)
    }

    async fn receive_json_rpc_response(&mut self) -> Result<JsonRpcResponse, ClientError> {
        let s = self.receive_string_with_u32_len().await
            .change_context(ClientError::Read)?;
        serde_json::from_str(&s)
            .change_context(ClientError::Parse)
    }

    async fn receive_server_event(&mut self) -> Result<ServerEvent, ClientError> {
        let s = self.receive_string_with_u32_len().await
            .change_context(ClientError::Read)?;
        serde_json::from_str(&s)
            .change_context(ClientError::Parse)
    }
}

impl <T: tokio::io::AsyncRead + Unpin> ConnectionUtilsRead for T {}

pub trait ConnectionUtilsWrite: tokio::io::AsyncWrite + Unpin  {
    async fn send_u8(&mut self, byte: u8) -> Result<(), ClientError> {
        self.write_u8(byte).await.change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_string_with_u32_len(&mut self, value: String) -> Result<(), ClientError> {
        let len_u32: u32 = TryInto::<u32>::try_into(value.len())
            .change_context(ClientError::UnsupportedStringLength)?;
        self.write_u32_le(len_u32)
            .await
            .change_context(ClientError::Write)?;
        self.write_all(value.as_bytes()).await.change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<(), ClientError> {
        let text = serde_json::to_string(&request)
            .change_context(ClientError::Serialize)?;
        self.send_string_with_u32_len(text).await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_response(
        &mut self,
        response: JsonRpcResponse
    ) -> Result<(), ClientError> {
        let text = serde_json::to_string(&response)
            .change_context(ClientError::Serialize)?;
        self.send_string_with_u32_len(text).await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_server_event(
        &mut self,
        server_event: &ServerEvent,
    ) -> Result<(), ClientError> {
        let text = serde_json::to_string(server_event)
            .change_context(ClientError::Serialize)?;
        self.send_string_with_u32_len(text).await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }
}

impl <T: tokio::io::AsyncWrite + Unpin> ConnectionUtilsWrite for T {}



pub struct ManagerClientWithRequestReceiver {
    pub(crate) client: ManagerClient,
    pub(crate) request_receiver: ManagerInstanceName,
}

pub trait RequestSenderCmds: Sized {
    fn request_receiver_name(&self) -> ManagerInstanceName;
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError>;

    async fn get_available_instances(
        self,
    ) -> Result<ManagerInstanceNameList, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetManagerInstanceNames,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::ManagerInstanceNames(info) = response.into_response() {
            Ok(info)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_secure_storage_encryption_key(
        self,
        key: ManagerInstanceName,
    ) -> Result<SecureStorageEncryptionKey, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSecureStorageEncryptionKey(key),
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SecureStorageEncryptionKey(key) = response.into_response() {
            Ok(key)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_system_info(
        self,
    ) -> Result<SystemInfo, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSystemInfo,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SystemInfo(info) = response.into_response() {
            Ok(info)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_software_update_status(
        self,
    ) -> Result<SoftwareUpdateStatus, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSoftwareUpdateStatus,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SoftwareUpdateStatus(status) = response.into_response() {
            Ok(status)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn trigger_software_update_task(
        self,
        task: SoftwareUpdateTaskType,
    ) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::TriggerSoftwareUpdateTask(task),
        );
        self.send_request(request).await?.require_successful()
    }

    async fn trigger_manual_task(
        self,
        task: ManualTaskType,
    ) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::TriggerManualTask(task),
        );
        self.send_request(request).await?.require_successful()
    }

    async fn get_scheduled_tasks_status(
        self,
    ) -> Result<ScheduledTaskStatus, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetScheduledTasksStatus,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::ScheduledTasksStatus(status) = response.into_response() {
            Ok(status)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn schedule_task(
        self,
        task: ScheduledTaskType,
        notify_backend: NotifyBackend,
    ) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::ScheduleTask(task, notify_backend),
        );
        self.send_request(request).await?.require_successful()
    }

    async fn unschedule_task(
        self,
        task: ScheduledTaskType,
    ) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::UnscheduleTask(task),
        );
        self.send_request(request).await?.require_successful()
    }
}

trait RpcResponseExtensions: Sized {
    fn require_successful(self) -> Result<(), ClientError>;
}

impl RpcResponseExtensions for JsonRpcResponse {
    fn require_successful(self) -> Result<(), ClientError> {
        if let JsonRpcResponseType::Successful = self.into_response() {
            Ok(())
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }
}

impl RequestSenderCmds for ManagerClientWithRequestReceiver {
    fn request_receiver_name(&self) -> ManagerInstanceName {
        self.request_receiver.clone()
    }
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        self.client.send_request(request).await
    }
}
