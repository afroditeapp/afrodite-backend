use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
pub struct Profile {
    name: String,
    /// Version used for caching profile in client side.
    version: Option<uuid::Uuid>,
    /// Profile visibility. Set true to make profile public.
    public: Option<bool>,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self { name, version: None, public: None }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> Option<uuid::Uuid> {
        self.version
    }

    pub fn remove_version(&mut self) {
        self.version.take();
    }

    pub fn generate_new_version(&mut self) {
        self.version = Some(uuid::Uuid::new_v4());
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or_default()
    }
}


// TODO: Create ProfileInternal and have all attributes there.
