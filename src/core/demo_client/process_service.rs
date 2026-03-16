use std::future::Future;

use sqlx::{Error, Postgres};

use crate::ResourceRecord;
use crate::core::application::{
    CreateResourceService,
    CreateResourceUow,
    DeleteResourceService,
    DeleteResourceUow,
};
use crate::core::domain::{Resource, ResourceId};
use crate::core::ports::UnitOfWorkPort;

#[derive(Debug, Clone)]
pub struct ClientResource {
    record: ResourceRecord
}

impl ClientResource {
    pub fn new(id: ResourceId, name: String, description: Option<String>) -> Self {
        Self { record: ResourceRecord::new(id, name, description) }
    }
}

impl Resource for ClientResource {
    fn id(&self) -> ResourceId {
        self.record.id()
    }
    
    fn name(&self) -> String {
        self.record.name()
    }

    fn description(&self) -> Option<String> {
        self.record.description()
    }
}

#[derive(Debug, Clone)]
pub struct DemoResourceRecord {
    pub resource: ClientResource,
    pub demo_name: String,
    pub demo_payload: String,
}

#[derive(Clone)]
pub struct DemoProcessService<U>
where
    U: UnitOfWorkPort<Db = Postgres, Error = Error>,
{
    core_create_service: CreateResourceService<CreateResourceUow<U>>,
    core_delete_service: DeleteResourceService<DeleteResourceUow<U>>,
}

impl<U> DemoProcessService<U>
where
    U: UnitOfWorkPort<Db = Postgres, Error = Error>,
{
    pub fn new(unit_of_work: U) -> Self {
        let create_uow = CreateResourceUow::new(unit_of_work.clone());
        let delete_uow = DeleteResourceUow::new(unit_of_work);

        Self {
            core_create_service: CreateResourceService::new(create_uow),
            core_delete_service: DeleteResourceService::new(delete_uow),
        }
    }

    pub fn create_demo_resource(
        &self,
        record: DemoResourceRecord,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        let resource_id = record.resource.id();
        let process_name = record.demo_name;
        let demo_payload = record.demo_payload;

        self.core_create_service.create_with(resource_id, move |tx| {
            Box::pin(async move {
                // Demo placeholder for client-specific algorithm executed in same tx.
                let _ = tx;
                let _ = process_name;
                let _ = demo_payload;

                Ok(())
            })
        })
    }

    pub fn delete_demo_resource(
        &self,
        record: DemoResourceRecord,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        let resource_id = record.resource.id();
        let demo_name = record.demo_name;
        let demo_payload = record.demo_payload;

        self.core_delete_service.delete_with(resource_id, move |tx| {
            Box::pin(async move {
                // Demo placeholder for client-specific algorithm executed in same tx.
                let _ = tx;
                let _ = demo_name;
                let _ = demo_payload;

                Ok(())
            })
        })
    }
}
