use model_profile::{GetProfileStatisticsResult, StatisticsProfileVisibility};

use tokio::sync::Mutex;

use server_data::{
    db_manager::RouterDatabaseReadHandle, result::Result, DataError
};

use crate::read::GetReadProfileCommands;

/// Cache publicly available profile statistics
#[derive(Debug, Default)]
pub struct ProfileStatisticsCache {
    pub data: Mutex<Option<GetProfileStatisticsResult>>,
}

impl ProfileStatisticsCache {
    const VISIBILITY: StatisticsProfileVisibility = StatisticsProfileVisibility::Public;
    pub async fn get_or_update_statistics(&self, handle: &RouterDatabaseReadHandle) -> Result<GetProfileStatisticsResult, DataError> {
        let mut data = self.data.lock().await;
        let r = match data.as_mut() {
            Some(data) => data.clone(),
            None => {
                let r = handle.profile().statistics().profile_statistics(Self::VISIBILITY).await?;
                *data = Some(r.clone());
                r
            }
        };
        Ok(r)
    }

    pub async fn update_statistics(&self, handle: &RouterDatabaseReadHandle) -> Result<GetProfileStatisticsResult, DataError> {
        let mut data = self.data.lock().await;
        let r = handle.profile().statistics().profile_statistics(Self::VISIBILITY).await?;
        *data = Some(r.clone());
        Ok(r)
    }
}
