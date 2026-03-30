use std::future::Future;
use crate::core::application::{CreateResourceUow, DeleteResourceUow};
use crate::core::domain::{Resource, ResourceId};
use crate::core::ports::{ResourceLifecycleWorkflowPort, UnitOfWorkPort};


#[derive(Clone)]
pub struct ResourceLifecycleWorkflow<CREATE, DELETE>
where
    CREATE: UnitOfWorkPort,
    DELETE: UnitOfWorkPort<Error = CREATE::Error>,
{
    create_uow: CreateResourceUow<CREATE>,
    delete_uow: DeleteResourceUow<DELETE>,
}

impl<CREATE, DELETE> ResourceLifecycleWorkflow<CREATE, DELETE>
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

impl<U> ResourceLifecycleWorkflow<U, U>
where
    U: UnitOfWorkPort,
{
    pub fn from_shared_uow(unit_of_work: U) -> Self {
        Self::new(unit_of_work.clone(), unit_of_work)
    }
}

impl<CREATE, DELETE> ResourceLifecycleWorkflowPort for ResourceLifecycleWorkflow<CREATE, DELETE>
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
    
    fn delete_resource(&self, resource_id: &ResourceId) -> impl Future<Output = Result<(), Self::Error>> + Send    {
        self.delete_uow.delete(resource)
    }
}