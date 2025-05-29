use std::collections::HashMap;

use model::{AccountId, UnixTime};
use model_server_data::{LocationIndexProfileData, ProfileAttributesInternal, ProfileLink, ProfileQueryMakerDetails};

#[derive(Debug)]
pub struct ProfilesAtLocation {
    profiles: HashMap<AccountId, LocationIndexProfileData>,
}

impl ProfilesAtLocation {
    pub fn new(account_id: AccountId, profile: LocationIndexProfileData) -> Self {
        let mut profiles = Self {
            profiles: HashMap::new(),
        };
        profiles.insert(account_id, profile);
        profiles
    }

    pub fn insert(&mut self, account_id: AccountId, profile: LocationIndexProfileData) {
        self.profiles.insert(account_id, profile);
    }

    pub fn remove(&mut self, account_id: &AccountId) -> Option<LocationIndexProfileData> {
        self.profiles.remove(account_id)
    }

    pub fn get(&self, account_id: &AccountId) -> Option<&LocationIndexProfileData> {
        self.profiles.get(account_id)
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub fn find_profiles(
        &self,
        query_maker_details: &ProfileQueryMakerDetails,
        attributes: Option<&ProfileAttributesInternal>,
        current_time: &UnixTime,
    ) -> Vec<ProfileLink> {
        self.profiles
            .values()
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
