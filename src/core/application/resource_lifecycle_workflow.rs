use std::future::Future;
use std::pin::Pin;

use crate::core::application::{CreateResourceUow, DeleteResourceUow};
use crate::core::domain::{Resource, ResourceId};
use crate::core::ports::{ResourceLifecycleWorkflowPort, UnitOfWorkPort};

#[derive(Clone)]
pub struct DefaultResourceLifecycleWorkflow<CREATE, DELETE>
where
    CREATE: UnitOfWorkPort,
    DELETE: UnitOfWorkPort<Error = CREATE::Error>,
{
    create_uow: CreateResourceUow<CREATE>,
    delete_uow: DeleteResourceUow<DELETE>,
}

impl<CREATE, DELETE> DefaultResourceLifecycleWorkflow<CREATE, DELETE>
where
    CREATE: UnitOfWorkPort,
    DELETE: UnitOfWorkPort<Error = CREATE::Error>,
{
    pub fn new(create_port: CREATE, delete_port: DELETE) -> Self {
        Self {
            create_uow: CreateResourceUow::new(create_port),
            delete_uow: DeleteResourceUow::new(delete_port),
        }
    }
}

impl<U> DefaultResourceLifecycleWorkflow<U, U>
where
    U: UnitOfWorkPort,
{
    pub fn from_shared_uow(unit_of_work: U) -> Self {
        Self::new(unit_of_work.clone(), unit_of_work)
    }
}

impl<CREATE, DELETE> ResourceLifecycleWorkflowPort for DefaultResourceLifecycleWorkflow<CREATE, DELETE>
where
    CREATE: UnitOfWorkPort,
    DELETE: UnitOfWorkPort<Error = CREATE::Error>,
{
    type Error = CREATE::Error;
    type CreateTx<'tx> = CREATE::Tx<'tx>;
    type DeleteTx<'tx> = DELETE::Tx<'tx>;

    fn create_resource<R>(&self, resource: &R) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.create_uow.create(resource)
    }

    fn create_resource_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.create_uow.create_by_id(resource_id)
    }

    fn create_resource_with<F>(&self, resource_id: ResourceId, client_algorithm: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::CreateTx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.create_uow.create_with(resource_id, client_algorithm)
    }

    fn delete_resource<R>(&self, resource: &R) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        R: Resource + Send + Sync,
    {
        self.delete_uow.delete(resource)
    }

    fn delete_resource_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.delete_uow.delete_by_id(resource_id)
    }

    fn delete_resource_with<F>(&self, resource_id: ResourceId, client_algorithm: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::DeleteTx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.delete_uow.delete_with(resource_id, client_algorithm)
    }
}