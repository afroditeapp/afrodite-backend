use model::ProfileLink;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MatchesPage {
    pub p: Vec<ProfileLink>,
}
