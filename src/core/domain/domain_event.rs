use std::{collections::BTreeMap, fmt::Display};

use serde::Serialize;

use crate::core::domain::ResourceId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DomainEvent {
    ComponentCreated(ComponentCreated),
    ComponentDeleted(ComponentDeleted),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentCreated {
    pub component_id: ResourceId,
    pub name: String,
    pub description: Option<String>,
    pub parent: Option<ResourceId>,
    pub children: Vec<ResourceId>,
    pub parameters: BTreeMap<String, String>,
}

impl Display for ComponentCreated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentCreated {{ id: {}, name: {} }}", self.component_id.to_string(), self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentDeleted {
    pub component_id: ResourceId,
}

