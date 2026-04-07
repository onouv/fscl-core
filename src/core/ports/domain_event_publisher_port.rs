use std::future::Future;

use crate::core::domain::DomainEvent;

pub trait DomainEventPublisherPort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn publish(
        &self,
        tx: &mut Self::Tx<'_>,
        event: &DomainEvent,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}