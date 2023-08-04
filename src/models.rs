

use diesel::{prelude::*, sqlite::Sqlite, deserialize::FromSql, sql_types::Binary, backend::Backend};

use crate::api::model::{AccountIdLight, ProfileVersion};



#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::Profile)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Profile {
    account_row_id: i64,
    pub version_uuid: ProfileVersion,
    location_key_x: i64,
    location_key_y: i64,
    pub name: String,
    pub profile_text: String,
}

impl FromSql<Binary, Sqlite> for AccountIdLight {
    fn from_sql(bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let uuid = uuid::Uuid::from_slice(&bytes)?;
        Ok(AccountIdLight::new(uuid))
    }
}

impl FromSql<Binary, Sqlite> for ProfileVersion {
    fn from_sql(bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let uuid = uuid::Uuid::from_slice(&bytes)?;
        Ok(ProfileVersion::new(uuid))
    }
}
