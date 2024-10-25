
use crate::statistics::ProfileStatisticsCache;

pub trait ProfileStatisticsCacheProvider {
    fn profile_statistics_cache(&self) -> &ProfileStatisticsCache;
}
