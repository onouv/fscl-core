use super::resource_id::ResourceId;

/// The essential data of any FSCL resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: ResourceId,
    pub name: String,
    pub description: Option<String>,
}

impl Resource {
    pub fn new(id: ResourceId, name: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            description,
        }
    }

    /*
        pub fn from_resource<R: Resource>(resource: &R) -> Self {
            Self {
                id: resource.id(),
                name: resource.name(),
                description: resource.description(),
            }
        }
    */
    pub fn id(&self) -> ResourceId {
        self.id.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }
}
