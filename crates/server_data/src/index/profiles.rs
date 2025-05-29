use std::collections::HashMap;

use model::{AccountId, UnixTime};
use model_server_data::{LocationIndexProfileData, ProfileAttributesInternal, ProfileLink, ProfileQueryMakerDetails};

#[derive(Debug)]
pub struct ProfilesAtLocation {
    men: HashMap<AccountId, LocationIndexProfileData>,
    women: HashMap<AccountId, LocationIndexProfileData>,
    non_binaries: HashMap<AccountId, LocationIndexProfileData>,
}

impl ProfilesAtLocation {
    pub fn new(account_id: AccountId, profile: LocationIndexProfileData) -> Self {
        let mut profiles = Self {
            men: HashMap::new(),
            women: HashMap::new(),
            non_binaries: HashMap::new(),
        };
        profiles.insert(account_id, profile);
        profiles
    }

    pub fn insert(&mut self, account_id: AccountId, profile: LocationIndexProfileData) {
        self.remove(&account_id);
        if profile.is_man() {
            self.men.insert(account_id, profile);
        } else if profile.is_woman() {
            self.women.insert(account_id, profile);
        } else {
            self.non_binaries.insert(account_id, profile);
        }
    }

    pub fn remove(&mut self, account_id: &AccountId) -> Option<LocationIndexProfileData> {
        self.men.remove(account_id)
            .or_else(|| self.women.remove(account_id))
            .or_else(|| self.non_binaries.remove(account_id))
    }

    pub fn get(&self, account_id: &AccountId) -> Option<&LocationIndexProfileData> {
        self.men.get(account_id)
            .or_else(|| self.women.get(account_id))
            .or_else(|| self.non_binaries.get(account_id))
    }

    pub fn len(&self) -> usize {
        self.men.len()
            .saturating_add(self.women.len())
            .saturating_add(self.non_binaries.len())
    }

    pub fn is_empty(&self) -> bool {
        self.men.is_empty() &&
        self.women.is_empty() &&
        self.non_binaries.is_empty()
    }

    pub fn find_profiles(
        &self,
        query_maker_details: &ProfileQueryMakerDetails,
        attributes: Option<&ProfileAttributesInternal>,
        current_time: &UnixTime,
    ) -> Vec<ProfileLink> {
        let men = if query_maker_details.search_groups_filter.is_searching_men() {
            Some(self.men.values())
        } else {
            None
        };
        let women = if query_maker_details.search_groups_filter.is_searching_women() {
            Some(self.women.values())
        } else {
            None
        };
        let non_binaries = if query_maker_details.search_groups_filter.is_searching_non_binaries() {
            Some(self.non_binaries.values())
        } else {
            None
        };

        men.into_iter().flatten()
            .chain(women.into_iter().flatten())
            .chain(non_binaries.into_iter().flatten())
            .filter(|p| {
                p.is_match(
                    query_maker_details,
                    attributes,
                    current_time,
                )
            })
            .map(|p| p.to_profile_link_value())
            .collect()
    }
}
