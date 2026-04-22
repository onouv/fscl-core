use fscl_messaging::EventEnvelope;

use crate::core::domain::DomainEvent;

pub trait DomainEventMapper: Clone + Send + Sync {
    fn map(&self, view_id: &str, event: &DomainEvent) -> EventEnvelope;
}
