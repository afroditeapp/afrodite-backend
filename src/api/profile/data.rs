use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

use crate::api::media::data::{ContentIdInternal, ContentId};


/// Profile's database data
#[derive(Debug, Clone)]
pub struct ProfileInternal {
    pub name: String,
    pub profile_text: String,
    pub public: bool,
    pub image1: Option<ContentId>,
    pub image2: Option<ContentId>,
    pub image3: Option<ContentId>,
    /// Version used for caching profile in client side.
    pub version_uuid: ProfileVersion,
}

/// Prfile for HTTP GET
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub profile_text: String,
    pub image1: Option<ContentId>,
    pub image2: Option<ContentId>,
    pub image3: Option<ContentId>,
    /// Version used for caching profile in client side.
    pub version: ProfileVersion,
}

impl Profile {
    pub fn into_update(self) -> ProfileUpdate {
        ProfileUpdate { profile_text: self.profile_text, image1: self.image1, image2: self.image2, image3: self.image3 }
    }
}

impl From<ProfileInternal> for Profile {
    fn from(value: ProfileInternal) -> Self {
        Self {
            name: value.name,
            profile_text: value.profile_text,
            image1: value.image1,
            image2: value.image2,
            image3: value.image3,
            version: value.version_uuid,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
pub struct ProfileUpdate {
    pub profile_text: String,
    pub image1: Option<ContentId>,
    pub image2: Option<ContentId>,
    pub image3: Option<ContentId>,
}

impl Profile {
    // pub fn new(name: String) -> Self {
    //     Self {
    //         name,
    //         version: None,
    //     }
    // }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct ProfileUpdateInternal {
    pub new_data: ProfileUpdate,
    /// Version used for caching profile in client side.
    pub version: ProfileVersion,
}

impl ProfileUpdateInternal {
    pub fn new(new_data: ProfileUpdate) -> Self {
        Self {
            new_data,
            version: ProfileVersion(uuid::Uuid::new_v4()),
        }
    }
}

// TODO: Create ProfileInternal and have all attributes there.

// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
// pub struct ProfileInternal {
//     profile: Profile,
//     /// Profile visibility. Set true to make profile public.
//     public: Option<bool>,
// }

// impl ProfileInternal {
//     pub fn new(name: String) -> Self {
//         Self {
//             profile: Profile::new(name),
//             public: None,
//         }
//     }

//     pub fn profile(&self) -> &Profile {
//         &self.profile
//     }

//     pub fn public(&self) -> bool {
//         self.public.unwrap_or_default()
//     }
// }

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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams, PartialEq, Eq, Hash, sqlx::Type)]
#[into_params(names("version_uuid"))]
#[sqlx(transparent)]
pub struct ProfileVersion(uuid::Uuid);
