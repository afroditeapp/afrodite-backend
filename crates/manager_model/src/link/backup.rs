use std::{io::{ErrorKind, Read}, num::Wrapping};

use simple_backend_utils::UuidBase64Url;

#[derive(Debug, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum BackupMessageType {
    /// Message without any data. Used for server connection testing.
    /// Sequence number is 0.
    Empty = 0,
    /// Backup source client sends this to backup target client.
    StartBackupSession = 1,
    /// Partial list of backend content. When empty content
    /// list is sent all Account IDs available are synced.
    ///
    /// Data contains list of
    ///
    /// - Account ID UUID (16 bytes, big-endian)
    /// - Content ID UUID count (u8)
    /// - Content ID UUID (16 bytes, big-endian)
    ContentList = 2,
    /// Target sends this to source if it does not have data for the content.
    ///
    /// Data:
    ///
    /// - Account ID UUID (16 bytes, big-endian)
    /// - Content ID UUID (16 bytes, big-endian)
    ContentQuery = 3,
    /// Source sends this to target when answering to content query.
    ///
    /// Data is content bytes or empty if there is some failure.
    ContentQueryAnswer = 4,
    /// When target is handled the received content list the target
    /// sends this to source.
    ContentListSyncDone = 5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupMessage {
    pub header: BackupMessageHeader,
    pub data: Vec<u8>,
}

impl BackupMessage {
    pub fn empty() -> Self {
        Self {
            header: BackupMessageHeader {
                message_type: BackupMessageType::Empty,
                backup_session: Wrapping(0),
            },
            data: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupMessageHeader {
    pub message_type: BackupMessageType,
    /// Backup session number
    pub backup_session: Wrapping<u32>,
}

pub struct AccountAndContent {
    pub account_id: UuidBase64Url,
    pub content_ids: Vec<UuidBase64Url>,
}

pub enum SourceToTargetMessage {
    StartBackupSession,
    ContentList {
        data: Vec<AccountAndContent>,
    },
    ContentQueryAnswer {
        data: Vec<u8>,
    },
}

impl SourceToTargetMessage {
    pub fn into_message(self, backup_session: u32) -> Result<BackupMessage, String> {
        let message_type = match self {
            Self::StartBackupSession => BackupMessageType::StartBackupSession,
            Self::ContentList { .. } => BackupMessageType::ContentList,
            Self::ContentQueryAnswer { .. } => BackupMessageType::ContentQueryAnswer,
        };

        let data = match self {
            Self::StartBackupSession => vec![],
            Self::ContentList { data } => {
                let mut serialized = vec![];
                for item in data {
                    serialized.extend(item.account_id.as_bytes());
                    let content_count: u8 = TryInto::<u8>::try_into(item.content_ids.len()).map_err(|e| e.to_string())?;
                    serialized.push(content_count);
                    for c in item.content_ids {
                        serialized.extend(c.as_bytes());
                    }
                }
                serialized
            }
            Self::ContentQueryAnswer { data } =>
                data,
        };

        Ok(BackupMessage {
            header: BackupMessageHeader {
                backup_session: Wrapping(backup_session),
                message_type,
            },
            data
        })
    }
}

impl TryFrom<BackupMessage> for SourceToTargetMessage {
    type Error = String;
    fn try_from(value: BackupMessage) -> Result<Self, Self::Error> {
        let m = match value.header.message_type {
            BackupMessageType::Empty |
            BackupMessageType::ContentListSyncDone |
            BackupMessageType::ContentQuery =>
                return Err(format!("Type conversion for message type {:?} is not supported", value.header.message_type)),
            BackupMessageType::StartBackupSession =>
                Self::StartBackupSession,
            BackupMessageType::ContentList => {
                let mut parsed = vec![];

                let mut data_reader = value.data.as_slice();

                loop {
                    let mut bytes = [0u8; 16];
                    match data_reader.read_exact(&mut bytes) {
                        Ok(()) => (),
                        Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(e.to_string()),
                    }
                    let account_id = UuidBase64Url::from_bytes(bytes);

                    let mut bytes = [0u8; 1];
                    data_reader.read_exact(&mut bytes)
                        .map_err(|e| e.to_string())?;
                    let content_count = bytes[0];

                    let mut content_ids = vec![];
                    for _ in 0..content_count {
                        let mut bytes = [0u8; 16];
                        data_reader.read_exact(&mut bytes)
                            .map_err(|e| e.to_string())?;
                        content_ids.push(UuidBase64Url::from_bytes(bytes));
                    }

                    parsed.push(AccountAndContent {
                        account_id,
                        content_ids,
                    });
                }

                SourceToTargetMessage::ContentList { data: parsed }
            }
            BackupMessageType::ContentQueryAnswer =>
                SourceToTargetMessage::ContentQueryAnswer { data: value.data },
        };

        Ok(m)
    }
}

pub enum TargetToSourceMessage {
    ContentListSyncDone,
    ContentQuery {
        account_id: UuidBase64Url,
        content_id: UuidBase64Url,
    }
}

impl TargetToSourceMessage {
    pub fn into_message(self, backup_session: u32) -> BackupMessage {
        let message_type = match self {
            Self::ContentListSyncDone => BackupMessageType::ContentListSyncDone,
            Self::ContentQuery { .. } => BackupMessageType::ContentQuery,
        };

        let data = match self {
            Self::ContentListSyncDone => vec![],
            Self::ContentQuery { account_id, content_id } =>
                account_id
                    .as_bytes()
                    .iter()
                    .chain(content_id.as_bytes())
                    .copied()
                    .collect::<Vec<u8>>()
        };

        BackupMessage {
            header: BackupMessageHeader {
                backup_session: Wrapping(backup_session),
                message_type,
            },
            data
        }
    }
}


impl TryFrom<BackupMessage> for TargetToSourceMessage {
    type Error = String;
    fn try_from(value: BackupMessage) -> Result<Self, Self::Error> {
        let m = match value.header.message_type {
            BackupMessageType::Empty |
            BackupMessageType::StartBackupSession |
            BackupMessageType::ContentList |
            BackupMessageType::ContentQueryAnswer =>
                return Err(format!("Type conversion for message type {:?} is not supported", value.header.message_type)),
            BackupMessageType::ContentListSyncDone =>
                Self::ContentListSyncDone,
            BackupMessageType::ContentQuery => {
                let mut data_reader = value.data.as_slice();

                let mut bytes = [0u8; 16];

                data_reader.read_exact(&mut bytes)
                    .map_err(|e| e.to_string())?;
                let account_id = UuidBase64Url::from_bytes(bytes);

                data_reader.read_exact(&mut bytes)
                    .map_err(|e| e.to_string())?;
                let content_id = UuidBase64Url::from_bytes(bytes);

                Self::ContentQuery {
                    account_id,
                    content_id
                }
            }
        };

        Ok(m)
    }
}
