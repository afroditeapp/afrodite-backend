use std::collections::HashMap;

use database::{define_history_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{ClientVersion, UnixTime};
use model_account::{ClientVersionCount, ClientVersionStatistics, GetClientVersionStatisticsResult, GetClientVersionStatisticsSettings};

define_history_read_commands!(HistoryReadAccountClientVersion);

impl HistoryReadAccountClientVersion<'_> {
    pub fn client_version_statistics(
        &mut self,
        settings: GetClientVersionStatisticsSettings,
    ) -> Result<GetClientVersionStatisticsResult, DieselDatabaseError> {
        use crate::schema::{
            history_client_version_statistics::dsl::*,
            history_client_version_statistics_save_time,
            history_client_version_statistics_version_number,
        };

        let max_time = settings.start_time.unwrap_or(UnixTime::new(i64::MAX));
        let min_time = settings.end_time.unwrap_or(UnixTime::new(0));

        let values: Vec<(UnixTime, i64, i64, i64, i64)> = history_client_version_statistics
            .inner_join(history_client_version_statistics_save_time::table.on(save_time_id.eq(history_client_version_statistics_save_time::id)))
            .inner_join(history_client_version_statistics_version_number::table.on(version_id.eq(history_client_version_statistics_version_number::id)))
            .filter(history_client_version_statistics_save_time::unix_time.le(max_time))
            .filter(history_client_version_statistics_save_time::unix_time.ge(min_time))
            .select((
                history_client_version_statistics_save_time::unix_time,
                history_client_version_statistics_version_number::major,
                history_client_version_statistics_version_number::minor,
                history_client_version_statistics_version_number::patch,
                count,
            ))
            .order((history_client_version_statistics_save_time::unix_time.desc(),))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        let mut version_data = HashMap::<ClientVersion, ClientVersionStatistics>::new();
        for (time, major, minor, patch, count_value) in values {
            let version = ClientVersion {
                major: major as u16,
                minor: minor as u16,
                patch: patch as u16,
            };
            let v = ClientVersionCount {
                t: time,
                c: count_value,
            };
            if let Some(statistics) = version_data.get_mut(&version) {
                statistics.values.push(v);
            } else {
                version_data.insert(version, ClientVersionStatistics { version, values: vec![v] });
            }
        }

        Ok(GetClientVersionStatisticsResult { values: version_data.into_values().collect::<Vec<_>>() })
    }
}
