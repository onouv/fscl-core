//! This module demonstrates how a client-specific service can be implemented by reusing
//! core application services and injecting a concrete Unit of Work.
//! It is not exported but ment only to demonstrate use of the core lib.

#![allow(dead_code)] // since this is a demo module, we may have unused code

use derive_getters::Getters;
use std::future::Future;

use crate::ResourceRecord;
use crate::core::application::ResourceLifecycleWorkflow;
use crate::core::domain::{Resource, ResourceId, ResourceIdError};
use crate::core::ports::{ResourceLifecycleWorkflowPort, UnitOfWorkPort};

#[derive(Debug, Clone, Getters)]
pub struct DemoResource {
    record: ResourceRecord,
    demo_data: String,
}

impl DemoResource {
    pub fn new(id: ResourceId, name: String, demo_data: String) -> Self {
        Self {
            record: ResourceRecord::new(id, name, None),
            demo_data,
        }
    }
}

impl Resource for DemoResource {
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
pub struct CreateDemoResourceRequest {
    pub id: String,
    pub name: String,
    pub data: String,
}

#[derive(Clone)]
pub struct DemoResourceService<W>
where
    W: ResourceLifecycleWorkflowPort,
{
    workflow: W,
}

impl<W> DemoResourceService<W>
where
    W: ResourceLifecycleWorkflowPort,
{
    pub fn new(workflow: W) -> Self {
        Self { workflow }
    }

    pub fn create_demo_resource(
        &self,
        request: CreateDemoResourceRequest,
    ) -> impl Future<Output = Result<(), W::Error>> + Send
    where
        W::Error: From<ResourceIdError>,
    {
        async move {
            let id = ResourceId::new(request.id.clone())?;
            let resource = DemoResource::new(id, request.name, request.data);
            // run the generic resource creation algorithm
            // on the shadow model and the messaging system
            self.workflow.create_resource(&resource).await
        }
    }

    pub fn delete_demo_resource(
        &self,
        id: &ResourceId 
    ) -> impl Future<Output = Result<(), W::Error>> + Send
    {
        async move {

            self.workflow.delete_resource(id).await
        }
    }
}

impl<U> DemoResourceService<ResourceLifecycleWorkflow<U, U>>
where
    U: UnitOfWorkPort,
{
    pub fn from_shared_uow(unit_of_work: U) -> Self {
        Self::new(ResourceLifecycleWorkflow::from_shared_uow(unit_of_work))
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use sqlx::Error;

    use super::{CreateDemoResourceRequest, DemoResource, DemoResourceService};
    use crate::core::adapters::driven::db::{DatabasePort, UnitOfWork};
    use crate::core::application::ResourceLifecycleWorkflow;
    use crate::core::domain::ResourceId;

    #[derive(Debug, Default)]
    struct MockTx;

    #[derive(Clone, Default)]
    struct MockDatabase {
        begin_calls: Arc<AtomicUsize>,
        commit_calls: Arc<AtomicUsize>,
        rollback_calls: Arc<AtomicUsize>,
    }

    impl MockDatabase {
        fn begin_call_count(&self) -> usize {
            self.begin_calls.load(Ordering::Relaxed)
        }

        fn commit_call_count(&self) -> usize {
            self.commit_calls.load(Ordering::Relaxed)
        }

        fn rollback_call_count(&self) -> usize {
            self.rollback_calls.load(Ordering::Relaxed)
        }
    }

    impl DatabasePort for MockDatabase {
        type Error = Error;
        type Tx<'tx> = MockTx;

        #[allow(clippy::manual_async_fn)] // since we need to apply trait bounds on future.
        fn execute_in_transaction<F>(
            &self,
            operation: F,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
        where
            F: for<'tx> FnOnce(
                    &'tx mut Self::Tx<'tx>,
                ) -> Pin<
                    Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>,
                > + Send
                + 'static,
        {
            async move {
                self.begin_calls.fetch_add(1, Ordering::Relaxed);

                let mut tx = MockTx;
                let result = operation(&mut tx).await;

                match result {
                    Ok(()) => {
                        self.commit_calls.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                    Err(err) => {
                        self.rollback_calls.fetch_add(1, Ordering::Relaxed);
                        Err(err)
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn demo_resource_service_can_be_instantiated_and_used_with_injected_uow() {
        // Emulate main composition: build concrete DB adapter, then inject the driven UoW.
        let mock_db = MockDatabase::default();
        let uow = UnitOfWork::new(mock_db.clone());
        let workflow = ResourceLifecycleWorkflow::from_shared_uow(uow);
        let service = DemoResourceService::new(workflow);

        let resource_id = ResourceId::new("demo-resource-1".to_string()).unwrap();
        let client_resource = DemoResource::new(
            resource_id,
            "Demo Name".to_string(),
            "Demo Data".to_string(),
        );

        let record = CreateDemoResourceRequest {
            resource: client_resource,
            id: "demo-resource".to_string(),
            name: "payload".to_string(),
        };

        service.create_demo_resource(record.clone()).await.unwrap();
        service.delete_demo_resource(record).await.unwrap();

        assert_eq!(mock_db.begin_call_count(), 2);
        assert_eq!(mock_db.commit_call_count(), 2);
        assert_eq!(mock_db.rollback_call_count(), 0);
    }
}
