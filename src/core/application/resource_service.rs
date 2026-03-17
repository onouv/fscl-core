use std::pin::Pin;

use crate::core::ports::UnitOfWorkPort;
use crate::core::domain::{Resource, ResourceId};

#[derive(Clone)]
/// Service to implement the standardized shadow-model-management algorithms. 
/// 
/// The `create_with` method enables clients to specify their own algorithm that will be 
/// executed after the core create algorithm.
/// 
/// The service relies on a Unit of Work port to manage transactions and ensure atomicity of 
/// operations.
pub struct ResourceService<CREATE, DELETE>
where
    CREATE: UnitOfWorkPort,
    DELETE: UnitOfWorkPort
{
    create_uow: CREATE,
    delete_uow: DELETE,
}

impl<CREATE, DELETE> ResourceService<CREATE, DELETE>
where
    CREATE: UnitOfWorkPort,
    DELETE: UnitOfWorkPort,
{
    pub fn new(create_uow: CREATE, delete_uow: DELETE) -> Self {
        Self { create_uow, delete_uow }
    }

    pub fn create<R>(&self, resource: &R) -> impl Future<Output = Result<(), CREATE::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.create_by_id(resource.id())
    }

    pub fn create_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), CREATE::Error>> + Send {
        self.create_with(resource_id, |_tx| Box::pin(async { Ok(()) }))
    }

    pub fn create_with<F>(
        &self,
        resource_id: ResourceId,
        client_algorithm: F,
    ) -> impl Future<Output = Result<(), CREATE::Error>> + Send
    where
        F: for<'tx> FnOnce(
            &'tx mut CREATE::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), CREATE::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.create_uow.execute(move |tx| {
            Box::pin(async move {
                // Standardized shadow-model-management create algorithm.
                // Replace with concrete operations as the algorithm evolves.
                let _ = tx;
                let _ = resource_id;
                client_algorithm(tx).await?;

                Ok(())
            })
        })
    }

    pub fn delete<R>(&self, resource: &R) -> impl Future<Output = Result<(), DELETE::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.delete_by_id(resource.id())
    }

    pub fn delete_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), DELETE::Error>> + Send {
        self.delete_with(resource_id, |_tx| Box::pin(async { Ok(()) }))
    }

    pub fn delete_with<F>(
        &self,
        resource_id: ResourceId,
        client_algorithm: F,
    ) -> impl Future<Output = Result<(), DELETE::Error>> + Send
    where
        F: for<'tx> FnOnce(
            &'tx mut DELETE::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), DELETE::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.delete_uow.execute(move |tx| {
            Box::pin(async move {
                // Standardized shadow-model-management delete algorithm.
                // Replace with concrete operations as the algorithm evolves.
                let _ = tx;
                let _ = resource_id;
                client_algorithm(tx).await?;

                Ok(())
            })
        })
    }
}
