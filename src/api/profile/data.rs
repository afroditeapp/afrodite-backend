use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use diesel::{prelude::*, sqlite::Sqlite, deserialize::FromSql, sql_types::Binary, backend::Backend};


use crate::api::model::AccountIdLight;
use crate::server::data::database::schema;

/// Profile's database data
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = schema::Profile)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProfileInternal {
    pub account_row_id: i64,
    pub version_uuid: ProfileVersion,
    pub location_key_x: i64,
    pub location_key_y: i64,
    pub name: String,
    pub profile_text: String,
}


// #[derive(Queryable, Selectable, Debug)]


// pub struct Profile {
//     account_row_id: i64,
//     pub version_uuid: ProfileVersion,
//     location_key_x: i64,
//     location_key_y: i64,
//     pub name: String,
//     pub profile_text: String,
// }




/// Prfile for HTTP GET
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub profile_text: String,
    /// Version used for caching profile in client side.
    pub version: ProfileVersion,
}

impl Profile {
    pub fn into_update(self) -> ProfileUpdate {
        ProfileUpdate {
            profile_text: self.profile_text,
        }
    }
}

impl From<ProfileInternal> for Profile {
    fn from(value: ProfileInternal) -> Self {
        Self {
            name: value.name,
            profile_text: value.profile_text,
            version: value.version_uuid,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
pub struct ProfileUpdate {
    pub profile_text: String,
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
            version: ProfileVersion {
                version_uuid: uuid::Uuid::new_v4(),
            },
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
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ProfilePage {
    pub profiles: Vec<ProfileLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileLink {
    id: AccountIdLight,
    version: ProfileVersion,
}

impl ProfileLink {
    pub fn new(id: AccountIdLight, profile: &ProfileInternal) -> Self {
        Self {
            id,
            version: profile.version_uuid,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams, PartialEq, Eq, Hash, diesel::FromSqlRow)]
pub struct ProfileVersion {
    version_uuid: uuid::Uuid,
}

impl ProfileVersion {
    pub fn new(version_uuid: uuid::Uuid) -> Self {
        Self { version_uuid }
    }

    pub fn as_uuid(&self) -> uuid::Uuid {
        self.version_uuid
    }
}

impl sqlx::Type<sqlx::Sqlite> for ProfileVersion {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <Uuid as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &<sqlx::Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <Uuid as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'a> sqlx::Encode<'a, sqlx::Sqlite> for ProfileVersion {
    fn encode_by_ref<'q>(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.version_uuid.encode_by_ref(buf)
    }

    fn encode<'q>(
        self,
        buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
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
    fn decode(
        value: <sqlx::Sqlite as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        <Uuid as sqlx::Decode<'_, sqlx::Sqlite>>::decode(value)
            .map(|id| ProfileVersion { version_uuid: id })
    }
}


impl FromSql<Binary, Sqlite> for ProfileVersion {
    fn from_sql(bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let uuid = uuid::Uuid::from_slice(&bytes)?;
        Ok(ProfileVersion::new(uuid))
    }
}
