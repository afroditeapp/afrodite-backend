use model::AccountId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ManualAssociationMembershipRegistry {
    pub registry: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ManualAssociationMembershipRegistryInput {
    pub registry: String,
}

/// Input for admin to change membership type of an existing entry.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateAssociationMembershipType {
    pub member: AccountId,
    pub membership_type: i16,
}
