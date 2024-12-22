use model_profile::{
    ProfileAgeCounts, ProfileStatisticsInternal, PublicProfileCounts, StatisticsGender, StatisticsProfileVisibility, UnixTime
};
use server_data::{define_cmd_wrapper_read, result::Result, DataError};

use crate::cache::CacheReadProfile;

define_cmd_wrapper_read!(ReadCommandsProfileStatistics);

impl ReadCommandsProfileStatistics<'_> {
    pub async fn profile_statistics(
        &self,
        profile_visibility: StatisticsProfileVisibility,
    ) -> Result<ProfileStatisticsInternal, DataError> {
        let generation_time = UnixTime::current_time();
        let mut account_count = 0;
        let mut public_profile_counts = PublicProfileCounts::default();

        let mut age_counts = ProfileAgeCounts::empty();

        self.read_cache_profile_and_common_for_all_accounts(|p, e| {
            account_count += 1;

            let visibility = e.account_state_related_shared_state.profile_visibility();

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
                        return;
                    }
                }
                StatisticsProfileVisibility::Private => {
                    if visibility.is_currently_public() {
                        return;
                    }
                }
            }

            if groups.is_man() {
                age_counts.increment_age(StatisticsGender::Man, p.data.age.value());
            } else if groups.is_woman() {
                age_counts.increment_age(StatisticsGender::Woman, p.data.age.value());
            } else if groups.is_non_binary() {
                age_counts.increment_age(StatisticsGender::NonBinary, p.data.age.value());
            }
        })
        .await?;

        Ok(ProfileStatisticsInternal::new(
            generation_time,
            age_counts,
            account_count,
            public_profile_counts,
        ))
    }
}
