//! This module demonstrates how a client-specific service can be implemented by reusing 
//! core application services and injecting a concrete Unit of Work.
//! It is not exported but ment only to demonstrate use of the core lib.

#![allow(dead_code)] // since this is a demo module, we may have unused code

use std::future::Future;
use crate::ResourceRecord;
use crate::core::application::DefaultResourceLifecycleWorkflow;
use crate::core::domain::{Resource, ResourceId};
use crate::core::ports::{ResourceLifecycleWorkflowPort, UnitOfWorkPort};

#[derive(Debug, Clone)]
pub struct DemoResource {
    record: ResourceRecord
}

impl DemoResource {
    pub fn new(id: ResourceId, name: String, description: Option<String>) -> Self {
        Self { record: ResourceRecord::new(id, name, description) }
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
pub struct DemoResourceRecord {
    pub resource: DemoResource,
    pub demo_name: String,
    pub demo_payload: String,
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
        record: DemoResourceRecord,
    ) -> impl Future<Output = Result<(), W::Error>> + Send {
        let resource_id = record.resource.id();
        let process_name = record.demo_name;
        let demo_payload = record.demo_payload;

        self.workflow.create_resource_with(resource_id, move |tx| {
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
    ) -> impl Future<Output = Result<(), W::Error>> + Send {
        let resource_id = record.resource.id();
        let demo_name = record.demo_name;
        let demo_payload = record.demo_payload;

        self.workflow.delete_resource_with(resource_id, move |tx| {
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

impl<U> DemoResourceService<DefaultResourceLifecycleWorkflow<U, U>>
where
    U: UnitOfWorkPort,
{
    pub fn from_shared_uow(unit_of_work: U) -> Self {
        Self::new(DefaultResourceLifecycleWorkflow::from_shared_uow(unit_of_work))
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use sqlx::Error;

    use super::{DemoResource, DemoResourceRecord, DemoResourceService};
    use crate::core::adapters::driven::db::{DatabasePort, UnitOfWork};
    use crate::core::application::DefaultResourceLifecycleWorkflow;
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
        fn execute_in_transaction<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
        where
            F: for<'tx> FnOnce(
                    &'tx mut Self::Tx<'tx>,
                ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
                + Send
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
        let workflow = DefaultResourceLifecycleWorkflow::from_shared_uow(uow);
        let service = DemoResourceService::new(workflow);

        let resource_id = ResourceId::new("demo-resource-1".to_string()).unwrap();
        let client_resource = DemoResource::new(
            resource_id,
            "Demo Name".to_string(),
            Some("Demo Description".to_string()),
        );

        let record = DemoResourceRecord {
            resource: client_resource,
            demo_name: "demo-resource".to_string(),
            demo_payload: "payload".to_string(),
        };

        service.create_demo_resource(record.clone()).await.unwrap();
        service.delete_demo_resource(record).await.unwrap();

        assert_eq!(mock_db.begin_call_count(), 2);
        assert_eq!(mock_db.commit_call_count(), 2);
        assert_eq!(mock_db.rollback_call_count(), 0);
    }
}
