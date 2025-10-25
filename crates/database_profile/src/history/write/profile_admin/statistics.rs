use database::{
    DieselDatabaseError, IntoDatabaseError, define_history_write_commands,
    history::write::GetDbHistoryWriteCommandsCommon,
};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{StatisticsSaveTimeId, UnixTime};
use model_profile::ProfileStatisticsInternal;

define_history_write_commands!(HistoryWriteProfileAdminStatistics);

impl<'a> HistoryWriteProfileAdminStatistics<'a> {
    pub fn save_statistics(
        &mut self,
        r: ProfileStatisticsInternal,
    ) -> Result<(), DieselDatabaseError> {
        let mut time_id = LazyTimeId::new(r.generation_time);
        self.save_count_if_needed_account(&mut time_id, r.account_count)?;
        self.save_count_if_needed_man(&mut time_id, r.public_profile_counts.men)?;
        self.save_count_if_needed_woman(&mut time_id, r.public_profile_counts.women)?;
        self.save_count_if_needed_non_binary(&mut time_id, r.public_profile_counts.nonbinaries)?;
        self.save_count_if_needed_all_genders(
            &mut time_id,
            r.public_profile_counts.men
                + r.public_profile_counts.women
                + r.public_profile_counts.nonbinaries,
        )?;

        type SaveMethod<'b> = fn(
            &mut HistoryWriteProfileAdminStatistics<'b>,
            &mut LazyTimeId,
            i16,
            i64,
        ) -> Result<(), DieselDatabaseError>;
        let mut handle_ages = |v: &Vec<i64>, save_method: SaveMethod<'a>| {
            for (i, c) in v.iter().enumerate() {
                let age = r.age_counts.start_age + i as i16;
                save_method(self, &mut time_id, age, *c)?
            }
            Ok::<(), error_stack::Report<DieselDatabaseError>>(())
        };

        handle_ages(&r.age_counts.men, Self::save_age_count_if_needed_man)?;
        handle_ages(&r.age_counts.women, Self::save_age_count_if_needed_woman)?;
        handle_ages(
            &r.age_counts.nonbinaries,
            Self::save_age_count_if_needed_non_binary,
        )?;

        let ages_all_genders = r
            .age_counts
            .men
            .iter()
            .zip(r.age_counts.women.iter())
            .zip(r.age_counts.nonbinaries.iter());

        for (i, ((c1, c2), c3)) in ages_all_genders.enumerate() {
            let age = r.age_counts.start_age + i as i16;
            let c = *c1 + *c2 + *c3;
            self.save_age_count_if_needed_all_genders(&mut time_id, age, c)?
        }

        Ok(())
    }
}

macro_rules! define_integer_change_method {
    (
        fn $method_name:ident,
        $table_name:ident,
    ) => {
        impl HistoryWriteProfileAdminStatistics<'_> {
            fn $method_name(
                &mut self,
                time_id_value: &mut LazyTimeId,
                count_value: i64,
            ) -> Result<(), DieselDatabaseError> {
                use crate::schema::$table_name::dsl::*;

                let latest = $table_name
                    .select(count)
                    .order(time_id.desc())
                    .first::<i64>(self.conn())
                    .optional()
                    .into_db_error(())?;

                if let Some(latest) = latest {
                    if latest == count_value {
                        return Ok(());
                    }
                }

                let time_id_value = time_id_value.time_id(self)?;

                insert_into($table_name)
                    .values((time_id.eq(time_id_value), count.eq(count_value)))
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

define_integer_change_method!(
    fn save_count_if_needed_all_genders,
    history_profile_statistics_count_changes_all_genders,
);

macro_rules! define_age_change_method {
    (
        fn $method_name:ident,
        $table_name:ident,
    ) => {
        impl HistoryWriteProfileAdminStatistics<'_> {
            fn $method_name(
                &mut self,
                time_id_value: &mut LazyTimeId,
                age_value: i16,
                count_value: i64,
            ) -> Result<(), DieselDatabaseError> {
                use crate::schema::$table_name::dsl::*;

                let latest = $table_name
                    .filter(age.eq(age_value))
                    .select(count)
                    .order(time_id.desc())
                    .first::<i64>(self.conn())
                    .optional()
                    .into_db_error(())?;

                if let Some(latest) = latest {
                    if latest == count_value {
                        return Ok(());
                    }
                }

                let time_id_value = time_id_value.time_id(self)?;

                insert_into($table_name)
                    .values((
                        time_id.eq(time_id_value),
                        age.eq(age_value),
                        count.eq(count_value),
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
    history_profile_statistics_age_changes_man,
);

define_age_change_method!(
    fn save_age_count_if_needed_woman,
    history_profile_statistics_age_changes_woman,
);

define_age_change_method!(
    fn save_age_count_if_needed_non_binary,
    history_profile_statistics_age_changes_non_binary,
);

define_age_change_method!(
    fn save_age_count_if_needed_all_genders,
    history_profile_statistics_age_changes_all_genders,
);

struct LazyTimeId {
    current_time: UnixTime,
    time_id: Option<StatisticsSaveTimeId>,
}

impl LazyTimeId {
    fn new(current_time: UnixTime) -> Self {
        Self {
            current_time,
            time_id: None,
        }
    }

    fn time_id(
        &mut self,
        cmds: &mut HistoryWriteProfileAdminStatistics,
    ) -> Result<StatisticsSaveTimeId, DieselDatabaseError> {
        if let Some(id) = self.time_id {
            Ok(id)
        } else {
            let id = cmds
                .write()
                .common_history()
                .get_or_create_save_time_id(self.current_time)?;
            self.time_id = Some(id);
            Ok(id)
        }
    }
}
