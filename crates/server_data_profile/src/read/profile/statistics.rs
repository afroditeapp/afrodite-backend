use model::{GetProfileStatisticsResult, ProfileAgesPage, PublicProfileCounts, StatisticsGender, StatisticsProfileVisibility, UnixTime};
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError
};

define_server_data_read_commands!(ReadCommandsProfileStatistics);
define_db_read_command!(ReadCommandsProfileStatistics);

impl<C: ReadCommandsProvider> ReadCommandsProfileStatistics<C> {
    pub async fn profile_statistics(
        &mut self,
        profile_visibility: StatisticsProfileVisibility,
    ) -> Result<GetProfileStatisticsResult, DataError> {

        let generation_time = UnixTime::current_time();
        let mut account_count = 0;
        let mut public_profile_counts = PublicProfileCounts::default();

        let mut man = ProfileAgesPage::empty(StatisticsGender::Man);
        let mut woman = ProfileAgesPage::empty(StatisticsGender::Woman);
        let mut non_binary = ProfileAgesPage::empty(StatisticsGender::NonBinary);

        self
            .cache().read_cache_for_all_accounts(|e| {
                account_count += 1;

                let visibility = e.account_state_related_shared_state.profile_visibility();
                let p = e.profile_data()?;

                let groups = p.state.search_group_flags;
                if visibility.is_currently_public() {
                    if groups.is_man() {
                        public_profile_counts.man += 1;
                    } else if groups.is_woman() {
                        public_profile_counts.woman += 1;
                    } else if groups.is_non_binary() {
                        public_profile_counts.non_binary += 1;
                    }
                }

                match profile_visibility {
                    StatisticsProfileVisibility::All => (),
                    StatisticsProfileVisibility::Public => {
                        if !visibility.is_currently_public() {
                            return Ok(());
                        }
                    }
                    StatisticsProfileVisibility::Private => {
                        if visibility.is_currently_public() {
                            return Ok(());
                        }
                    }
                }

                if groups.is_man() {
                    man.increment_age(p.data.age.value());
                } else if groups.is_woman() {
                    woman.increment_age(p.data.age.value());
                } else if groups.is_non_binary() {
                    non_binary.increment_age(p.data.age.value());
                }

                Ok(())
            })
            .await
            .into_data_error(())?;

        Ok(GetProfileStatisticsResult {
            generation_time,
            profile_ages: vec![
                man,
                woman,
                non_binary,
            ],
            account_count,
            public_profile_counts,
        })
    }
}
