use std::collections::BTreeMap;

use thiserror::Error;

use crate::{Resource, core::domain::{ComponentCreated, ComponentDeleted, DomainEvent, ResourceId}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    inner: Resource, 
    parent: Option<ResourceId>,
    children: Vec<ResourceId>,
    parameters: BTreeMap<String, String>,
}

impl Component {
    pub fn create(
        id: ResourceId,
        name: String,
        description: Option<String>,
        parent: Option<ResourceId>,
        children: Vec<ResourceId>,
        parameters: BTreeMap<String, String>,
    ) -> Result<(Self, DomainEvent), ComponentError> {
        if name.trim().is_empty() {
            return Err(ComponentError::NameEmpty);
        }

        let component = Self {
            inner: Resource { id, name, description },
            parent,
            children,
            parameters,
        };

        Ok((component.clone(), component.created_event()))
    }

    pub fn delete(self) -> Result<Vec<DomainEvent>, ComponentError> {

        // TODO: inform our parent and any dependents, delete our Children

        Ok(vec![DomainEvent::ComponentDeleted(ComponentDeleted { component_id: self.inner.id.clone() })])
    }

    /*pub fn deleted_event_for(component_id: ResourceId) -> DomainEvent {
        DomainEvent::ComponentDeleted(ComponentDeleted { component_id })
    }*/

    pub fn id(&self) -> &ResourceId {
        &self.inner.id
    }

    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    fn created_event(&self) -> DomainEvent {
        DomainEvent::ComponentCreated(ComponentCreated {
            component_id: self.inner.id.clone(),
            name: self.inner.name.clone(),
            description: self.inner.description.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
            parameters: self.parameters.clone(),
        })
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ComponentError {
    #[error("Component name cannot be empty.")]
    NameEmpty,

    #[error("Component cannot delete.")]
    CannotDelete
}