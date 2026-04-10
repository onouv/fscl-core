use super::event_message::EventMessage;
use crate::core::domain::{ComponentCreated, ComponentDeleted, DomainEvent};

impl From<DomainEvent> for EventMessage {
    fn from(domain_event: DomainEvent) -> Self {
        match domain_event {
            DomainEvent::ComponentCreated(evt) => component_created_event_to_message(evt),
            DomainEvent::ComponentDeleted(evt) => component_deleted_event_to_message(evt), 
        }
    }
}

fn component_created_event_to_message(event: ComponentCreated) -> EventMessage {
    EventMessage::default()
        .with_event_type("created")
        .with_aggregate_type("component")
        .with_aggregate_id(event.component_id.as_str())
        .with_view_id("component_view")
        .with_payload(event).unwrap()
}

fn component_deleted_event_to_message(event: ComponentDeleted) -> EventMessage {
    EventMessage::default()
        .with_event_type("deleted")
        .with_aggregate_type("component")
        .with_aggregate_id(event.component_id.as_str())
        .with_view_id("component_view")
        .with_payload(event).unwrap()
}