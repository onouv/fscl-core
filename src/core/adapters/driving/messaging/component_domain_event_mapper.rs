use chrono::Utc;
use fscl_messaging::{AggregateType, EventEnvelope};

use crate::core::build_event_envelope;
use crate::core::domain::{ComponentCreated, ComponentDeleted, DomainEvent};

use super::DomainEventMapper;

#[derive(Clone, Default)]
pub struct ComponentDomainEventMapper;

impl DomainEventMapper for ComponentDomainEventMapper {
    fn map(&self, view_id: &str, event: &DomainEvent) -> EventEnvelope {
        match event {
            DomainEvent::ComponentCreated(evt) => component_created_event_to_envelope(view_id, evt),
            DomainEvent::ComponentDeleted(evt) => component_deleted_event_to_envelope(view_id, evt),
        }
    }
}

fn component_created_event_to_envelope(view_id: &str, event: &ComponentCreated) -> EventEnvelope {
    build_event_envelope(
        Utc::now(),
        "created",
        AggregateType::Component,
        &event.component_id,
        view_id,
        event,
    )
    .expect("component created event should serialize")
}

fn component_deleted_event_to_envelope(view_id: &str, event: &ComponentDeleted) -> EventEnvelope {
    build_event_envelope(
        Utc::now(),
        "deleted",
        AggregateType::Component,
        &event.component_id,
        view_id,
        event,
    )
    .expect("component deleted event should serialize")
}
