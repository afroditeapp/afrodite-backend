use database::{define_history_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{GetProfileStatisticsHistoryResult, ProfileStatisticsHistoryValue, ProfileStatisticsHistoryValueTypeInternal, StatisticsGender};

define_history_read_commands!(HistoryReadProfileAdminStatistics, HistorySyncReadProfileAdminStatistics);

impl<C: ConnectionProvider> HistorySyncReadProfileAdminStatistics<C> {
    pub fn profile_statistics_history(
        &mut self,
        settings: ProfileStatisticsHistoryValueTypeInternal,
    ) -> Result<GetProfileStatisticsHistoryResult, DieselDatabaseError> {
        use ProfileStatisticsHistoryValueTypeInternal as S;
        let values = match settings {
            S::Accounts => self.count_changes_account(),
            S::Public { gender: None } => self.count_changes_all_genders(),
            S::Public { gender: Some(StatisticsGender::Man) } => self.count_changes_man(),
            S::Public { gender: Some(StatisticsGender::Woman) } => self.count_changes_woman(),
            S::Public { gender: Some(StatisticsGender::NonBinary) } => self.count_changes_non_binary(),
            S::AgeChange { gender: None, age } => self.age_changes_all_genders(age),
            S::AgeChange { gender: Some(StatisticsGender::Man), age } => self.age_changes_man(age),
            S::AgeChange { gender: Some(StatisticsGender::Woman), age } => self.age_changes_woman(age),
            S::AgeChange { gender: Some(StatisticsGender::NonBinary), age } => self.age_changes_non_binary(age),
        }?;

        Ok(GetProfileStatisticsHistoryResult {
            values
        })
    }
}

macro_rules! define_read_count_change_methods {
    (
        fn $method_name:ident,
        $table_name:ident,
    ) => {
        impl<C: ConnectionProvider> HistorySyncReadProfileAdminStatistics<C> {
            fn $method_name(
                &mut self,
            ) -> Result<Vec<ProfileStatisticsHistoryValue>, DieselDatabaseError> {
                use crate::schema::{
                    history_profile_statistics_save_time::dsl::*,
                    $table_name::dsl::*
                };

                $table_name
                    .inner_join(history_profile_statistics_save_time)
                    .select((
                        unix_time,
                        count,
                    ))
                    .order((
                        unix_time.desc(),
                    ))
                    .load(self.conn())
                    .change_context(DieselDatabaseError::Execute)
            }
        }
    };
}

define_read_count_change_methods!(
    fn count_changes_account,
    history_profile_statistics_count_changes_account,
);

define_read_count_change_methods!(
    fn count_changes_man,
    history_profile_statistics_count_changes_man,
);

define_read_count_change_methods!(
    fn count_changes_woman,
    history_profile_statistics_count_changes_woman,
);

define_read_count_change_methods!(
    fn count_changes_non_binary,
    history_profile_statistics_count_changes_non_binary,
);

define_read_count_change_methods!(
    fn count_changes_all_genders,
    history_profile_statistics_count_changes_all_genders,
);

macro_rules! define_read_age_change_methods {
    (
        fn $method_name:ident,
        $table_name:ident,
    ) => {
        impl<C: ConnectionProvider> HistorySyncReadProfileAdminStatistics<C> {
            fn $method_name(
                &mut self,
                age_value: i64,
            ) -> Result<Vec<ProfileStatisticsHistoryValue>, DieselDatabaseError> {
                use crate::schema::{
                    history_profile_statistics_save_time::dsl::*,
                    $table_name::dsl::*
                };

                $table_name
                    .inner_join(history_profile_statistics_save_time)
                    .filter(age.eq(age_value))
                    .select((
                        unix_time,
                        count,
                    ))
                    .order((
                        unix_time.desc(),
                    ))
                    .load(self.conn())
                    .change_context(DieselDatabaseError::Execute)
            }
        }
    };
}

define_read_age_change_methods!(
    fn age_changes_man,
    history_profile_statistics_age_changes_men,
);

define_read_age_change_methods!(
    fn age_changes_woman,
    history_profile_statistics_age_changes_woman,
);

define_read_age_change_methods!(
    fn age_changes_non_binary,
    history_profile_statistics_age_changes_non_binary,
);

define_read_age_change_methods!(
    fn age_changes_all_genders,
    history_profile_statistics_age_changes_all_genders,
);

/*

In theory it is possible to calculate ProfileStatisticsHistoryValueTypeInternal::Profile
and ProfileStatisticsHistoryValueTypeInternal::Age from gender specific tables.
However the code for that is not trivial, so it was decided that
data for those are stored in DB.

struct RemoveOldestUnixTimeIterator {
    man: vec::IntoIter<ProfileStatisticsHistoryValue>,
    woman: vec::IntoIter<ProfileStatisticsHistoryValue>,
    non_binary: vec::IntoIter<ProfileStatisticsHistoryValue>,
    current_man: Option<ProfileStatisticsHistoryValue>,
    current_woman: Option<ProfileStatisticsHistoryValue>,
    current_non_binary: Option<ProfileStatisticsHistoryValue>,
}

impl RemoveOldestUnixTimeIterator {
    pub fn new(
        man: Vec<ProfileStatisticsHistoryValue>,
        woman: Vec<ProfileStatisticsHistoryValue>,
        non_binary: Vec<ProfileStatisticsHistoryValue>,
    ) -> Self {
        Self {
            man: man.into_iter(),
            woman: woman.into_iter(),
            non_binary: non_binary.into_iter(),
            current_man: None,
            current_woman: None,
            current_non_binary: None,
        }
    }
}

impl Iterator for RemoveOldestUnixTimeIterator {
    type Item = (
        ProfileStatisticsHistoryValue,
        ProfileStatisticsHistoryValue,
        ProfileStatisticsHistoryValue,
    );
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_man.is_none() && self.current_woman.is_none() && self.current_non_binary.is_none() {
            self.current_man = self.man.next();
            self.current_woman = self.woman.next();
            self.current_non_binary = self.non_binary.next();
            return match (self.current_man, self.current_woman, self.current_non_binary) {
                (Some(m), Some(w), Some(non_binary)) => Some((m, w, non_binary)),
                _ => None,
            };
        }

        match (self.current_man, self.current_woman, self.current_non_binary) {
            (Some(m), Some(w), Some(non_binary)) => {

            }
            _ => None,
        }
    }
}

*/
