use model_profile::ProfileStatisticsInternal;
use server_data::{
    db_manager::RouterDatabaseReadHandle, result::Result, statistics::ProfileStatisticsCache,
    DataError,
};

use crate::read::GetReadProfileCommands;

pub trait ProfileStatisticsCacheUtils {
    async fn get_or_update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
    ) -> Result<ProfileStatisticsInternal, DataError>;
    async fn update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
    ) -> Result<ProfileStatisticsInternal, DataError>;
}

impl ProfileStatisticsCacheUtils for ProfileStatisticsCache {
    async fn get_or_update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
    ) -> Result<ProfileStatisticsInternal, DataError> {
        let mut data = self.data.lock().await;
        let r = match data.as_mut() {
            Some(data) => data.clone(),
            None => {
                let r = handle
                    .profile()
                    .statistics()
                    .profile_statistics(Self::VISIBILITY)
                    .await?;
                *data = Some(r.clone());
                r
            }
        };
        Ok(r)
    }

    async fn update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
    ) -> Result<ProfileStatisticsInternal, DataError> {
        let mut data = self.data.lock().await;
        let r = handle
            .profile()
            .statistics()
            .profile_statistics(Self::VISIBILITY)
            .await?;
        *data = Some(r.clone());
        Ok(r)
    }
}
