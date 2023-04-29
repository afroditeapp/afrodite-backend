use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use uuid::Uuid;

use crate::api::media::data::{ContentId};


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
            version: ProfileVersion { version_uuid: uuid::Uuid::new_v4() },
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams, PartialEq, Eq, Hash)]
pub struct ProfileVersion {
    version_uuid: uuid::Uuid
}

impl sqlx::Type<sqlx::Sqlite> for ProfileVersion {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <Uuid as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &<sqlx::Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <Uuid as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl <'a> sqlx::Encode<'a, sqlx::Sqlite> for ProfileVersion {
    fn encode_by_ref<'q>(&self, buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
        self.version_uuid.encode_by_ref(buf)
    }

    fn encode<'q>(self, buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        self.version_uuid.encode_by_ref(buf)
    }

    fn produces(&self) -> Option<<sqlx::Sqlite as sqlx::Database>::TypeInfo> {
        <Uuid as sqlx::Encode<'a, sqlx::Sqlite>>::produces(&self.version_uuid)
    }

    fn size_hint(&self) -> usize {
        self.version_uuid.size_hint()
    }
}

impl sqlx::Decode<'_, sqlx::Sqlite> for ProfileVersion {
    fn decode(value: <sqlx::Sqlite as sqlx::database::HasValueRef<'_>>::ValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        <Uuid as sqlx::Decode<'_, sqlx::Sqlite>>::decode(value).map(|id| ProfileVersion { version_uuid: id})
    }
}
