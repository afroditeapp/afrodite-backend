use std::num::Wrapping;


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
    /// Source sends this to target when answering to content query.
    ///
    /// Data is content bytes or empty if there is some failure.
    ContentQueryAnswer = 3,
    /// Target sends this to source if it does not have data for the content.
    ///
    /// Data:
    ///
    /// - Account ID UUID (16 bytes, big-endian)
    /// - Content ID UUID (16 bytes, big-endian)
    ContentQuery = 4,
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
