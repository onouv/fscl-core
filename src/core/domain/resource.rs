use std::fmt::{self, Display, Formatter};
use thiserror::Error;

pub trait Resource {
    fn id(&self) -> ResourceId;
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