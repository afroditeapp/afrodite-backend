use std::{
    io::{ErrorKind, Read},
    num::Wrapping,
};

use simple_backend_model::ContentQualityVariant;
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
    /// - Content quality variant bitflag (1 byte): High=1, Medium=2, Low=4
    ContentQuery = 3,
    /// Source sends this to target when answering to content query.
    ///
    /// Content data is empty when error occurs.
    ///
    /// Data:
    ///
    /// - Repeated for each variant:
    ///   - Content quality variant byte (1 byte): High=1, Medium=2, Low=4
    ///   - Content SHA-256 (32 bytes)
    ///   - Content data length (4 bytes, little-endian, u32)
    ///   - Content data
    ContentQueryAnswer = 4,
    /// When target is handled the received content list the target
    /// sends this to source.
    ContentListSyncDone = 5,
    /// Start file backup transfer. Backup source client sends this
    /// after the last [BackupMessageType::ContentList]. When file name
    /// is empty all files are backuped.
    ///
    /// Data:
    ///
    /// - File SHA-256 (32 bytes)
    /// - File name UTF-8 bytes
    StartFileBackup = 6,
    /// File backup data package. Empty package means that transfer is
    /// completed.
    ///
    /// Data:
    ///
    /// - Package number (u32, little-endian, can wrap)
    /// - Data
    FileBackupData = 7,
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

pub type ContentQueryAnswer = Vec<(ContentQualityVariant, Sha256Bytes, Vec<u8>)>;

pub enum SourceToTargetMessage {
    StartBackupSession,
    ContentList {
        data: Vec<AccountAndContent>,
    },
    ContentQueryAnswer(ContentQueryAnswer),
    StartFileBackup {
        sha256: Sha256Bytes,
        file_name: String,
    },
    FileBackupData {
        package_number: Wrapping<u32>,
        data: Vec<u8>,
    },
}

impl SourceToTargetMessage {
    pub fn into_message(self, backup_session: u32) -> Result<BackupMessage, String> {
        let message_type = match self {
            Self::StartBackupSession => BackupMessageType::StartBackupSession,
            Self::ContentList { .. } => BackupMessageType::ContentList,
            Self::ContentQueryAnswer(_) => BackupMessageType::ContentQueryAnswer,
            Self::StartFileBackup { .. } => BackupMessageType::StartFileBackup,
            Self::FileBackupData { .. } => BackupMessageType::FileBackupData,
        };

        let data = match self {
            Self::StartBackupSession => vec![],
            Self::ContentList { data } => {
                let mut serialized = vec![];
                for item in data {
                    serialized.extend(item.account_id.as_bytes());
                    let content_count: u8 = TryInto::<u8>::try_into(item.content_ids.len())
                        .map_err(|e| e.to_string())?;
                    serialized.push(content_count);
                    for c in item.content_ids {
                        serialized.extend(c.as_bytes());
                    }
                }
                serialized
            }
            Self::ContentQueryAnswer(v) => {
                let mut serialized = vec![];
                for (variant, sha256, data) in v {
                    serialized.push(variant.as_u8());
                    serialized.extend(sha256.0);
                    let len: u32 =
                        TryInto::<u32>::try_into(data.len()).map_err(|e| e.to_string())?;
                    serialized.extend(len.to_le_bytes());
                    serialized.extend(data);
                }
                serialized
            }
            Self::StartFileBackup { sha256, file_name } => sha256
                .0
                .iter()
                .chain(file_name.as_bytes())
                .copied()
                .collect(),
            Self::FileBackupData {
                package_number,
                data,
            } => package_number
                .0
                .to_le_bytes()
                .into_iter()
                .chain(data)
                .collect(),
        };

        Ok(BackupMessage {
            header: BackupMessageHeader {
                backup_session: Wrapping(backup_session),
                message_type,
            },
            data,
        })
    }
}

impl TryFrom<BackupMessage> for SourceToTargetMessage {
    type Error = String;
    fn try_from(value: BackupMessage) -> Result<Self, Self::Error> {
        let m = match value.header.message_type {
            BackupMessageType::Empty
            | BackupMessageType::ContentListSyncDone
            | BackupMessageType::ContentQuery => {
                return Err(format!(
                    "Type conversion for message type {:?} is not supported",
                    value.header.message_type
                ));
            }
            BackupMessageType::StartBackupSession => Self::StartBackupSession,
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
                    data_reader
                        .read_exact(&mut bytes)
                        .map_err(|e| e.to_string())?;
                    let content_count = bytes[0];

                    let mut content_ids = vec![];
                    for _ in 0..content_count {
                        let mut bytes = [0u8; 16];
                        data_reader
                            .read_exact(&mut bytes)
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
            BackupMessageType::ContentQueryAnswer => {
                let mut data_reader = value.data.as_slice();
                let mut variants = vec![];

                loop {
                    let mut variant_byte = [0u8; 1];
                    match data_reader.read_exact(&mut variant_byte) {
                        Ok(()) => (),
                        Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(e.to_string()),
                    }
                    let Some(variant) = ContentQualityVariant::from_u8(variant_byte[0]) else {
                        return Err("Invalid content quality variant byte".to_string());
                    };
                    let mut sha256 = [0u8; 32];
                    data_reader
                        .read_exact(&mut sha256)
                        .map_err(|e| e.to_string())?;
                    let mut len_bytes = [0u8; 4];
                    data_reader
                        .read_exact(&mut len_bytes)
                        .map_err(|e| e.to_string())?;
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    let mut d = vec![0u8; len];
                    data_reader.read_exact(&mut d).map_err(|e| e.to_string())?;
                    variants.push((variant, Sha256Bytes(sha256), d));
                }

                SourceToTargetMessage::ContentQueryAnswer(variants)
            }
            BackupMessageType::StartFileBackup => {
                let Some((sha256, file_name)) = value.data.split_at_checked(32) else {
                    return Err("No enough data".to_string());
                };
                let sha256: [u8; 32] =
                    TryInto::<[u8; 32]>::try_into(sha256).map_err(|v| v.to_string())?;
                let file_name = String::from_utf8(file_name.to_vec()).map_err(|e| e.to_string())?;
                SourceToTargetMessage::StartFileBackup {
                    sha256: Sha256Bytes(sha256),
                    file_name,
                }
            }
            BackupMessageType::FileBackupData => {
                let Some((package_number, data)) = value.data.split_at_checked(4) else {
                    return Err("No enough data".to_string());
                };
                let package_number =
                    TryInto::<[u8; 4]>::try_into(package_number).map_err(|e| e.to_string())?;
                let package_number = Wrapping(u32::from_le_bytes(package_number));
                let data = data.to_vec();
                SourceToTargetMessage::FileBackupData {
                    package_number,
                    data,
                }
            }
        };

        Ok(m)
    }
}

pub enum TargetToSourceMessage {
    ContentListSyncDone,
    ContentQuery {
        account_id: UuidBase64Url,
        content_id: UuidBase64Url,
        high: bool,
        medium: bool,
        low: bool,
    },
}

impl TargetToSourceMessage {
    pub fn into_message(self, backup_session: u32) -> BackupMessage {
        let message_type = match self {
            Self::ContentListSyncDone => BackupMessageType::ContentListSyncDone,
            Self::ContentQuery { .. } => BackupMessageType::ContentQuery,
        };

        let data = match self {
            Self::ContentListSyncDone => vec![],
            Self::ContentQuery {
                account_id,
                content_id,
                high,
                medium,
                low,
            } => {
                let mut bitflags: u8 = 0;
                if high {
                    bitflags |= ContentQualityVariant::High.as_u8();
                }
                if medium {
                    bitflags |= ContentQualityVariant::Medium.as_u8();
                }
                if low {
                    bitflags |= ContentQualityVariant::Low.as_u8();
                }
                account_id
                    .as_bytes()
                    .iter()
                    .chain(content_id.as_bytes())
                    .copied()
                    .chain([bitflags])
                    .collect::<Vec<u8>>()
            }
        };

        BackupMessage {
            header: BackupMessageHeader {
                backup_session: Wrapping(backup_session),
                message_type,
            },
            data,
        }
    }
}

impl TryFrom<BackupMessage> for TargetToSourceMessage {
    type Error = String;
    fn try_from(value: BackupMessage) -> Result<Self, Self::Error> {
        let m = match value.header.message_type {
            BackupMessageType::Empty
            | BackupMessageType::StartBackupSession
            | BackupMessageType::ContentList
            | BackupMessageType::ContentQueryAnswer
            | BackupMessageType::StartFileBackup
            | BackupMessageType::FileBackupData => {
                return Err(format!(
                    "Type conversion for message type {:?} is not supported",
                    value.header.message_type
                ));
            }
            BackupMessageType::ContentListSyncDone => Self::ContentListSyncDone,
            BackupMessageType::ContentQuery => {
                let mut data_reader = value.data.as_slice();

                let mut bytes = [0u8; 16];

                data_reader
                    .read_exact(&mut bytes)
                    .map_err(|e| e.to_string())?;
                let account_id = UuidBase64Url::from_bytes(bytes);

                data_reader
                    .read_exact(&mut bytes)
                    .map_err(|e| e.to_string())?;
                let content_id = UuidBase64Url::from_bytes(bytes);

                let mut bitflag = [0u8; 1];
                data_reader
                    .read_exact(&mut bitflag)
                    .map_err(|e| e.to_string())?;
                let bitflag = bitflag[0];

                Self::ContentQuery {
                    account_id,
                    content_id,
                    high: bitflag & ContentQualityVariant::High.as_u8() != 0,
                    medium: bitflag & ContentQualityVariant::Medium.as_u8() != 0,
                    low: bitflag & ContentQualityVariant::Low.as_u8() != 0,
                }
            }
        };

        Ok(m)
    }
}

pub struct Sha256Bytes(pub [u8; 32]);

impl Sha256Bytes {
    pub fn to_shasum_tool_compatible_checksum(&self, file_name: &str) -> String {
        format!(
            "{} *{}\n",
            base16ct::lower::encode_string(&self.0),
            file_name,
        )
    }
}
