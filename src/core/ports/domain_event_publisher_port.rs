use std::future::Future;

use crate::core::domain::DomainEvent;

pub trait DomainEventPublisherPort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    /*  Publish a domain event to the transactional outbox within the given transaction.
        Notice that an implementation must provide the appropriate view identifier in the 
        published artefact.
     */
    fn publish(
        &self,
        tx: &mut Self::Tx<'_>,
        event: &DomainEvent,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}