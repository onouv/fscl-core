use std::future::Future;
use std::pin::Pin;

use sqlx::Transaction;

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
}

impl<P: UnitOfWorkPort> UnitOfWorkPort for CreateResourceUow<P> {
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
/// Service to implement the standardized shadow-model-management create algorithm. 
/// 
/// The `create_with` method enables clients to specify their own algorithm that will be 
/// executed after the core create algorithm.
/// 
/// The service relies on a Unit of Work port to manage transactions and ensure atomicity of 
/// operations.
pub struct CreateResourceService<U>
where
    U: UnitOfWorkPort,
{
    uow: U,
}

impl<U> CreateResourceService<U>
where
    U: UnitOfWorkPort,
{
    pub fn new(uow: U) -> Self {
        Self { uow }
    }

    pub fn create<R>(&self, resource: &R) -> impl Future<Output = Result<(), U::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.create_by_id(resource.id())
    }

    pub fn create_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), U::Error>> + Send {
        self.create_with(resource_id, |_tx| Box::pin(async { Ok(()) }))
    }

    pub fn create_with<F>(
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
                Self::run_core_create_shadow_model_algorithm(tx, &resource_id).await?;
                client_algorithm(tx).await?;

                Ok(())
            })
        })
    }

    fn run_core_create_shadow_model_algorithm<'tx>(
        tx: &'tx mut Transaction<'_, U::Db>,
        resource_id: &'tx ResourceId,
    ) -> Pin<Box<dyn Future<Output = Result<(), U::Error>> + Send + 'tx>> {
        Box::pin(async move {
            // Standardized shadow-model-management create algorithm.
            // Replace with concrete sqlx operations as the algorithm evolves.
            let _ = tx;
            let _ = resource_id;

            Ok(())
        })
    }
}
