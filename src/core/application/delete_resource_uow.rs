use std::future::Future;
use std::pin::Pin;

use crate::core::domain::{Resource, ResourceId};
use crate::core::ports::UnitOfWorkPort;

#[derive(Clone)]
pub struct DeleteResourceUow<P: UnitOfWorkPort> {
    port: P,
}

impl<P: UnitOfWorkPort> DeleteResourceUow<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn port(&self) -> &P {
        &self.port
    }

    pub fn delete<R>(&self, resource: &R) -> impl Future<Output = Result<(), P::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.delete_by_id(resource.id())
    }

    pub fn delete_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), P::Error>> + Send {
        self.delete_with(resource_id, |_tx| Box::pin(async { Ok(()) }))
    }

    pub fn delete_with<F>(
        &self,
        resource_id: ResourceId,
        client_algorithm: F,
    ) -> impl Future<Output = Result<(), P::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut P::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), P::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.port.execute(move |tx| {
            Box::pin(async move {
                // Standardized shadow-model-management delete algorithm.
                let _ = tx;
                let _ = resource_id;

                client_algorithm(tx).await?;
                Ok(())
            })
        })
    }
}

impl<P: UnitOfWorkPort> UnitOfWorkPort for DeleteResourceUow<P> {
    type Error = P::Error;
    type Tx<'tx> = P::Tx<'tx>;

    fn execute<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.port.execute(operation)
    }
}
