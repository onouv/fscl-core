use std::fmt::{self, Display, Formatter};
use thiserror::Error;

pub trait Resource {
    fn id(&self) -> ResourceId;
    fn name(&self) -> String;
    fn description(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRecord {
    pub id: ResourceId,
    pub name: String,
    pub description: Option<String>,
}

impl ResourceRecord {
    pub fn new(id: ResourceId, name: String, description: Option<String>) -> Self {
        Self { id, name, description }
    }

    pub fn from_resource<R: Resource>(resource: &R) -> Self {
        Self {
            id: resource.id(),
            name: resource.name(),
            description: resource.description(),
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn new(id: String) -> Result<Self, ResourceIdError> {
        if id.is_empty() {
            return Err(ResourceIdError::ResourceIdEmpty);
        }

        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Error)]
pub enum ResourceIdError {
    ResourceIdEmpty,
    // There will more parsing errors...
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for ResourceIdError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let result = match self {
            ResourceIdError::ResourceIdEmpty => "Empty id string."
        }; 
        
        write!(f, "{}", result)
    }
}