use model_server_data::ProfileStatisticsInternal;
use tokio::sync::Mutex;

/// Cache profile statistics
#[derive(Debug, Default)]
pub struct ProfileStatisticsCache {
    pub data: Mutex<Option<ProfileStatisticsInternal>>,
}
