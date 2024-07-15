
use chrono::NaiveDate;
use diesel::prelude::*;
use crate::SetAccountSetup;

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
)]
pub struct SetProfileSetup {
    pub birthdate: NaiveDate,
}


impl From<SetAccountSetup> for SetProfileSetup {
    fn from(value: SetAccountSetup) -> Self {
        Self {
            birthdate: value.birthdate
        }
    }
}

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::profile_setup)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileSetup {
    birthdate: Option<NaiveDate>,
}
