use std::future::Future;
use std::pin::Pin;

use sqlx::Transaction;

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
}

impl<P: UnitOfWorkPort> UnitOfWorkPort for DeleteResourceUow<P> {
    type Db = P::Db;
    type Error = P::Error;

    fn execute<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Transaction<'_, Self::Db>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send,
    {
        self.port.execute(operation)
    }
}

#[derive(Clone)]
pub struct DeleteResourceService<U>
where
    U: UnitOfWorkPort,
{
    uow: U,
}

impl<U> DeleteResourceService<U>
where
    U: UnitOfWorkPort,
{
    pub fn new(uow: U) -> Self {
        Self { uow }
    }

    pub fn delete<R>(&self, resource: &R) -> impl Future<Output = Result<(), U::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.delete_by_id(resource.id())
    }

    pub fn delete_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), U::Error>> + Send {
        self.delete_with(resource_id, |_tx| Box::pin(async { Ok(()) }))
    }

    pub fn delete_with<F>(
        &self,
        resource_id: ResourceId,
        client_algorithm: F,
    ) -> impl Future<Output = Result<(), U::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Transaction<'_, U::Db>,
            ) -> Pin<Box<dyn Future<Output = Result<(), U::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.uow.execute(move |tx| {
            Box::pin(async move {
                Self::run_core_delete_shadow_model_algorithm(tx, &resource_id).await?;
                client_algorithm(tx).await?;

                Ok(())
            })
        })
    }

    fn run_core_delete_shadow_model_algorithm<'tx>(
        tx: &'tx mut Transaction<'_, U::Db>,
        resource_id: &'tx ResourceId,
    ) -> Pin<Box<dyn Future<Output = Result<(), U::Error>> + Send + 'tx>> {
        Box::pin(async move {
            // Standardized shadow-model-management delete algorithm.
            // Replace with concrete sqlx operations as the algorithm evolves.
            let _ = tx;
            let _ = resource_id;

            Ok(())
        })
    }
}
