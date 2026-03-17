use std::future::Future;
use std::pin::Pin;

use crate::core::domain::{Resource, ResourceId};
use crate::core::ports::UnitOfWorkPort;

#[derive(Clone)]
pub struct CreateResourceUow<P: UnitOfWorkPort> {
    port: P,
}

impl<P: UnitOfWorkPort> CreateResourceUow<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn port(&self) -> &P {
        &self.port
    }

    pub fn create<R>(&self, resource: &R) -> impl Future<Output = Result<(), P::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.create_by_id(resource.id())
    }

    pub fn create_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), P::Error>> + Send {
        self.create_with(resource_id, |_tx| Box::pin(async { Ok(()) }))
    }

    pub fn create_with<F>(
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
                // Standardized shadow-model-management create algorithm.
                let _ = tx;
                let _ = resource_id;

                client_algorithm(tx).await?;
                Ok(())
            })
        })
    }
}

impl<P: UnitOfWorkPort> UnitOfWorkPort for CreateResourceUow<P> {
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

