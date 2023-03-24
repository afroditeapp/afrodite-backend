use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::model::AccountIdLight;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
pub struct Profile {
    name: String,
    /// Version used for caching profile in client side.
    version: Option<uuid::Uuid>,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self { name, version: None }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> Option<uuid::Uuid> {
        self.version
    }

    pub fn remove_version(&mut self) {
        self.version.take();
    }

    pub fn generate_new_version(&mut self) {
        self.version = Some(uuid::Uuid::new_v4());
    }


}


// TODO: Create ProfileInternal and have all attributes there.


#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ProfileInternal {
    profile: Profile,
    /// Profile visibility. Set true to make profile public.
    public: Option<bool>,
}

impl ProfileInternal {
    pub fn new(name: String) -> Self {
        Self { profile: Profile::new(name), public: None }
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or_default()
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ProfilePage {
    latitude: Vec<ProfileLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileLink {
    id: uuid::Uuid,
    version: uuid::Uuid,
}
