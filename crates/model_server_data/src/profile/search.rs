use model::NextNumberStorage;
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
