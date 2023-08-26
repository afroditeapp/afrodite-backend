use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use diesel::{deserialize::FromSql, prelude::*, serialize::ToSql, sql_types::Binary};
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{macros::diesel_uuid_wrapper, AccountId, AccountIdDb};

/// Profile's database data
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::profile)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileInternal {
    pub account_id: AccountIdDb,
    pub version_uuid: ProfileVersion,
    pub location_key_x: i64,
    pub location_key_y: i64,
    pub name: String,
    pub profile_text: String,
}

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
    id: AccountId,
    version: ProfileVersion,
}

impl ProfileLink {
    pub fn new(id: AccountId, profile: &ProfileInternal) -> Self {
        Self {
            id,
            version: profile.version_uuid,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    Hash,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct ProfileVersion {
    version_uuid: uuid::Uuid,
}

impl ProfileVersion {
    pub fn new(version_uuid: uuid::Uuid) -> Self {
        Self { version_uuid }
    }

    pub fn new_random() -> Self {
        let version_uuid = uuid::Uuid::new_v4();
        Self { version_uuid }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.version_uuid
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

diesel_uuid_wrapper!(ProfileVersion);

// impl<DB: Backend> FromSql<Binary, DB> for ProfileVersion
// where
//     Vec<u8>: FromSql<Binary, DB>,
// {
//     fn from_sql(
//         bytes: <DB as diesel::backend::Backend>::RawValue<'_>,
//     ) -> diesel::deserialize::Result<Self> {
//         let bytes = Vec::<u8>::from_sql(bytes)?;
//         let uuid = uuid::Uuid::from_slice(&bytes)?;
//         Ok(ProfileVersion::new(uuid))
//     }
// }

// impl<DB: Backend> ToSql<Binary, DB> for ProfileVersion
// where
//     [u8]: ToSql<Binary, DB>,
// {
//     fn to_sql<'b>(
//         &'b self,
//         out: &mut diesel::serialize::Output<'b, '_, DB>,
//     ) -> diesel::serialize::Result {
//         self.as_uuid().as_bytes().to_sql(out)
//     }
// }

#[derive(Debug, Hash, PartialEq, Clone, Copy, Default, Eq)]
pub struct LocationIndexKey {
    pub y: u16,
    pub x: u16,
}

impl LocationIndexKey {
    pub fn x(&self) -> usize {
        self.x as usize
    }

    pub fn y(&self) -> usize {
        self.y as usize
    }
}

#[derive(Debug)]
pub struct CellData {
    pub next_up: AtomicU16,
    pub next_down: AtomicU16,
    pub next_left: AtomicU16,
    pub next_right: AtomicU16,
    pub profiles_in_this_area: AtomicBool,
}

impl std::ops::Index<LocationIndexKey> for DMatrix<CellData> {
    type Output = <Self as std::ops::Index<(usize, usize)>>::Output;

    fn index(&self, key: LocationIndexKey) -> &Self::Output {
        &self[(key.y as usize, key.x as usize)]
    }
}

impl CellData {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            next_down: AtomicU16::new(height.checked_sub(1).unwrap()),
            next_up: AtomicU16::new(0),
            next_left: AtomicU16::new(0),
            next_right: AtomicU16::new(width.checked_sub(1).unwrap()),
            profiles_in_this_area: AtomicBool::new(false),
        }
    }

    pub fn next_down(&self) -> usize {
        self.next_down.load(Ordering::Relaxed) as usize
    }

    pub fn next_up(&self) -> usize {
        self.next_up.load(Ordering::Relaxed) as usize
    }

    pub fn next_left(&self) -> usize {
        self.next_left.load(Ordering::Relaxed) as usize
    }

    pub fn next_right(&self) -> usize {
        self.next_right.load(Ordering::Relaxed) as usize
    }

    pub fn profiles(&self) -> bool {
        self.profiles_in_this_area.load(Ordering::Relaxed)
    }

    pub fn set_next_down(&self, i: usize) {
        self.next_down.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_up(&self, i: usize) {
        self.next_up.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_left(&self, i: usize) {
        self.next_left.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_right(&self, i: usize) {
        self.next_right.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_profiles(&self, value: bool) {
        self.profiles_in_this_area.store(value, Ordering::Relaxed)
    }
}
