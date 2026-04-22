use std::future::Future;

use crate::core::domain::DomainEvent;
use crate::core::ports::DomainEventPublisherPort;

use super::{DomainEventMapper, OutboxWriter};

#[derive(Clone)]
pub struct DomainEventOutboxPublisher<M, W> {
    view_id: String,
    mapper: M,
    writer: W,
}

impl<M, W> DomainEventOutboxPublisher<M, W>
where
    M: DomainEventMapper,
    W: OutboxWriter,
{
    pub fn new(view_id: impl Into<String>, mapper: M, writer: W) -> Self {
        Self {
            view_id: view_id.into(),
            mapper,
            writer,
        }
    }
}

impl<M, W> DomainEventPublisherPort for DomainEventOutboxPublisher<M, W>
where
    M: DomainEventMapper,
    W: OutboxWriter,
{
    type Error = W::Error;
    type Tx<'tx> = W::Tx<'tx>;

    fn publish(
        &self,
        tx: &mut Self::Tx<'_>,
        event: &DomainEvent,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let envelope = self.mapper.map(&self.view_id, event);
        self.writer.append(tx, envelope)
    }
}
