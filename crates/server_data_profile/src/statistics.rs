use std::sync::Arc;

use model_profile::{ProfileStatisticsInternal, UnixTime};
use server_data::{
    DataError, db_manager::RouterDatabaseReadHandle, result::Result,
    statistics::ProfileStatisticsCache,
};
use simple_backend::perf::PerfMetricsManagerData;

use crate::read::GetReadProfileCommands;

pub trait ProfileStatisticsCacheUtils {
    async fn get_or_update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
        perf_data: Arc<PerfMetricsManagerData>,
    ) -> Result<ProfileStatisticsInternal, DataError>;
    async fn update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
        perf_data: Arc<PerfMetricsManagerData>,
    ) -> Result<ProfileStatisticsInternal, DataError>;
}

impl ProfileStatisticsCacheUtils for ProfileStatisticsCache {
    async fn get_or_update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
        perf_data: Arc<PerfMetricsManagerData>,
    ) -> Result<ProfileStatisticsInternal, DataError> {
        let mut data = self.data.lock().await;

        if let Some(data) = data.as_ref()
            && UnixTime::current_time() < data.generation_time.add_seconds(60 * 15)
        {
            return Ok(data.clone());
        }

        let r = handle
            .profile()
            .statistics()
            .profile_statistics(Self::VISIBILITY, perf_data)
            .await?;
        *data = Some(r.clone());
        Ok(r)
    }

    async fn update_statistics(
        &self,
        handle: &RouterDatabaseReadHandle,
        perf_data: Arc<PerfMetricsManagerData>,
    ) -> Result<ProfileStatisticsInternal, DataError> {
        let mut data = self.data.lock().await;
        let r = handle
            .profile()
            .statistics()
            .profile_statistics(Self::VISIBILITY, perf_data)
            .await?;
        *data = Some(r.clone());
        Ok(r)
    }
}
