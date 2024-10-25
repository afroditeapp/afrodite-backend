
use database::{define_history_write_commands, ConnectionProvider, DieselDatabaseError, IntoDatabaseError};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{GetProfileStatisticsResult, SaveTimeId, StatisticsGender, UnixTime};

define_history_write_commands!(HistoryWriteProfileAdminStatistics, HistorySyncWriteProfileAdminStatistics);

impl<C: ConnectionProvider> HistorySyncWriteProfileAdminStatistics<C> {
    pub fn save_statistics(
        &mut self,
        r: GetProfileStatisticsResult,
    ) -> Result<(), DieselDatabaseError> {
        let time_id = self.save_time(r.generation_time)?;
        self.save_count_if_needed_account(time_id, r.account_count)?;
        self.save_count_if_needed_man(time_id, r.public_profile_counts.man)?;
        self.save_count_if_needed_woman(time_id, r.public_profile_counts.woman)?;
        self.save_count_if_needed_non_binary(time_id, r.public_profile_counts.non_binary)?;

        for v in r.profile_ages {
            let save_method = match v.gender {
                StatisticsGender::Man => Self::save_age_count_if_needed_man,
                StatisticsGender::Woman => Self::save_age_count_if_needed_woman,
                StatisticsGender::NonBinary => Self::save_age_count_if_needed_non_binary,
            };
            for (i, c) in v.profile_counts.iter().enumerate() {
                let age = v.start_age + i as i64;
                save_method(self, time_id, age, *c)?
            }
        }

        Ok(())
    }

    fn save_time(
        &mut self,
        time: UnixTime
    ) -> Result<SaveTimeId, DieselDatabaseError> {
        use crate::schema::history_profile_statistics_save_time::dsl::*;

        insert_into(history_profile_statistics_save_time)
            .values(unix_time.eq(time))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())
    }


}

macro_rules! define_integer_change_method {
    (
        fn $method_name:ident,
        $table_name:ident,
    ) => {
        impl<C: ConnectionProvider> HistorySyncWriteProfileAdminStatistics<C> {
            fn $method_name(
                &mut self,
                time_id: SaveTimeId,
                count_value: i64,
            ) -> Result<(), DieselDatabaseError> {
                use crate::schema::$table_name::dsl::*;

                let latest = $table_name
                    .select(count)
                    .order(save_time_id.desc())
                    .first::<i64>(self.conn())
                    .into_db_error(())?;

                if latest == count_value {
                    return Ok(());
                }

                insert_into($table_name)
                    .values((
                        save_time_id.eq(time_id),
                        count.eq(count_value),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;

                Ok(())
            }
        }
    };
}

define_integer_change_method!(
    fn save_count_if_needed_account,
    history_profile_statistics_count_changes_account,
);

define_integer_change_method!(
    fn save_count_if_needed_man,
    history_profile_statistics_count_changes_man,
);

define_integer_change_method!(
    fn save_count_if_needed_woman,
    history_profile_statistics_count_changes_woman,
);

define_integer_change_method!(
    fn save_count_if_needed_non_binary,
    history_profile_statistics_count_changes_non_binary,
);

macro_rules! define_age_change_method {
    (
        fn $method_name:ident,
        $table_name:ident,
    ) => {
        impl<C: ConnectionProvider> HistorySyncWriteProfileAdminStatistics<C> {
            fn $method_name(
                &mut self,
                time_id: SaveTimeId,
                age_value: i64,
                count_value: i64,
            ) -> Result<(), DieselDatabaseError> {
                use crate::schema::$table_name::dsl::*;

                let latest = $table_name
                    .filter(age.eq(age_value))
                    .select(count)
                    .order(save_time_id.desc())
                    .first::<i64>(self.conn())
                    .into_db_error(())?;

                if latest == count_value {
                    return Ok(());
                }

                insert_into($table_name)
                    .values((
                        save_time_id.eq(time_id),
                        age.eq(age_value),
                        count.eq(count_value)
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;

                Ok(())
            }
        }
    };
}

define_age_change_method!(
    fn save_age_count_if_needed_man,
    history_profile_statistics_age_changes_men,
);

define_age_change_method!(
    fn save_age_count_if_needed_woman,
    history_profile_statistics_age_changes_woman,
);

define_age_change_method!(
    fn save_age_count_if_needed_non_binary,
    history_profile_statistics_age_changes_non_binary,
);
