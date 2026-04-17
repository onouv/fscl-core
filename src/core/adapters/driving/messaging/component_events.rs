use chrono::Utc;
use fscl_messaging::{AggregateType, EventEnvelope};
use crate::core::build_event_envelope;
use crate::core::domain::{ComponentCreated, ComponentDeleted, DomainEvent};

impl From<DomainEvent> for EventEnvelope {
    fn from(domain_event: DomainEvent) -> Self {
        match domain_event {
            DomainEvent::ComponentCreated(evt) => component_created_event_to_envelope(evt),
            DomainEvent::ComponentDeleted(evt) => component_deleted_event_to_envelope(evt), 
        }
    }
}

fn component_created_event_to_envelope(event: ComponentCreated) -> EventEnvelope {
    let aggregate_id = event.component_id.clone();

    build_event_envelope(
        Utc::now(),
        "created",
        AggregateType::Component,
        &aggregate_id,
        "component_view",
        event,
    )
    .expect("component created event should serialize")
}

fn component_deleted_event_to_envelope(event: ComponentDeleted) -> EventEnvelope {
    let aggregate_id = event.component_id.clone();

    build_event_envelope(
        Utc::now(),
        "deleted",
        AggregateType::Component,
        &aggregate_id,
        "component_view",
        event,
    )
    .expect("component deleted event should serialize")
}