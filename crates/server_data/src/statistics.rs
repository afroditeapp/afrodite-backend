use model_server_data::{ProfileStatisticsInternal, StatisticsProfileVisibility};
use tokio::sync::Mutex;

/// Cache publicly available profile statistics
#[derive(Debug, Default)]
pub struct ProfileStatisticsCache {
    pub data: Mutex<Option<ProfileStatisticsInternal>>,
}

impl ProfileStatisticsCache {
    pub const VISIBILITY: StatisticsProfileVisibility = StatisticsProfileVisibility::Public;
}
