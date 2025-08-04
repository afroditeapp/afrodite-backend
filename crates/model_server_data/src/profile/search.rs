use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
};
use model::{NextNumberStorage, SelectedWeekdays};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutomaticProfileSearchIteratorSessionIdInternal {
    id: i64,
}

impl AutomaticProfileSearchIteratorSessionIdInternal {
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AutomaticProfileSearchIteratorSessionId {
    id: i64,
}

impl From<AutomaticProfileSearchIteratorSessionIdInternal>
    for AutomaticProfileSearchIteratorSessionId
{
    fn from(value: AutomaticProfileSearchIteratorSessionIdInternal) -> Self {
        Self { id: value.id }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::profile_automatic_profile_search_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct AutomaticProfileSearchSettings {
    pub new_profiles: bool,
    pub attribute_filters: bool,
    pub distance_filters: bool,
    pub weekdays: SelectedWeekdays,
}

impl Default for AutomaticProfileSearchSettings {
    fn default() -> Self {
        Self {
            new_profiles: false,
            attribute_filters: false,
            distance_filters: false,
            weekdays: SelectedWeekdays::all(),
        }
    }
}
