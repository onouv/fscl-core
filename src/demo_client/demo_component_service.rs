//! Demo client service for component create/delete using the core application slice.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::future::Future;

use crate::core::application::{
    ComponentLifecycleUow, CreateComponentError, CreateComponentRequest, DeleteComponentError,
    DeleteComponentRequest,
};
use crate::core::ports::{ComponentRepositoryPort, DomainEventPublisherPort, UnitOfWorkPort};

static VIEW_ID: &str = "demo";

#[derive(Debug, Clone)]
pub struct CreateDemoComponentRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DeleteDemoComponentRequest {
    pub id: String,
}

#[derive(Clone)]
pub struct DemoComponentService<U, R, S>
where
    U: UnitOfWorkPort,
    R: ComponentRepositoryPort<Error = U::Error>,
    S: DomainEventPublisherPort<Error = U::Error>,
{
    uow: ComponentLifecycleUow<U, R, S>,
}

impl<U, R, S> DemoComponentService<U, R, S>
where
    U: UnitOfWorkPort,
    R: ComponentRepositoryPort<Error = U::Error> + 'static,
    S: DomainEventPublisherPort<Error = U::Error> + 'static,
{
    pub fn new(lifecycle: ComponentLifecycleUow<U, R, S>) -> Self {
        Self { uow: lifecycle }
    }

    pub fn create_demo_component(
        &self,
        request: CreateDemoComponentRequest,
    ) -> impl Future<Output = Result<(), CreateComponentError<U::Error>>> + Send
    where
        R: for<'tx> ComponentRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
        S: for<'tx> DomainEventPublisherPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        self.uow.create_component(CreateComponentRequest {
            id: request.id,
            name: request.name,
            description: request.description,
            parent: request.parent,
            children: request.children,
            parameters: request.parameters,
        })
    }

    pub fn delete_demo_component(
        &self,
        request: DeleteDemoComponentRequest,
    ) -> impl Future<Output = Result<(), DeleteComponentError<U::Error>>> + Send
    where
        R: for<'tx> ComponentRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
        S: for<'tx> DomainEventPublisherPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        self.uow
            .delete_component(DeleteComponentRequest { id: request.id })
    }
}

#[cfg(test)]
mod tests {
    use std::future::{Future, ready};
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use fscl_messaging::EventEnvelope;

    use super::{CreateDemoComponentRequest, DeleteDemoComponentRequest, DemoComponentService};
    use crate::core::adapters::driving::messaging::{
        ComponentDomainEventMapper, DomainEventOutboxPublisher, OutboxWriter,
    };
    use crate::core::application::ComponentLifecycleUow;
    use crate::core::ports::{ComponentRepositoryPort, UnitOfWorkPort};
    use crate::demo_client::demo_component_service::VIEW_ID;
    use crate::{Component, IdFormat, ResourceId};

    #[derive(Clone, Default)]
    struct MockTx;

    #[derive(Clone, Default)]
    struct MockUow {
        execute_calls: Arc<AtomicUsize>,
    }

    impl MockUow {
        fn execute_call_count(&self) -> usize {
            self.execute_calls.load(Ordering::Relaxed)
        }
    }

    impl UnitOfWorkPort for MockUow {
        type Error = String;
        type Tx<'tx> = MockTx;

        fn execute<T, F>(
            &self,
            operation: F,
        ) -> impl Future<Output = Result<T, Self::Error>> + Send + '_
        where
            T: Send,
            F: for<'tx> FnOnce(
                    &'tx mut Self::Tx<'tx>,
                ) -> Pin<
                    Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'tx>,
                > + Send
                + 'static,
        {
            async move {
                self.execute_calls.fetch_add(1, Ordering::Relaxed);
                let mut tx = MockTx;
                operation(&mut tx).await
            }
        }
    }

    #[derive(Clone, Default)]
    struct MockRepo {
        component: Arc<Mutex<Option<Component>>>,
        find_calls: Arc<AtomicUsize>,
        upsert_calls: Arc<AtomicUsize>,
        delete_calls: Arc<AtomicUsize>,
    }

    impl MockRepo {
        fn with_component(component: Component) -> Self {
            Self {
                component: Arc::new(Mutex::new(Some(component))),
                find_calls: Arc::new(AtomicUsize::new(0)),
                upsert_calls: Arc::new(AtomicUsize::new(0)),
                delete_calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn find_call_count(&self) -> usize {
            self.find_calls.load(Ordering::Relaxed)
        }

        fn upsert_call_count(&self) -> usize {
            self.upsert_calls.load(Ordering::Relaxed)
        }

        fn delete_call_count(&self) -> usize {
            self.delete_calls.load(Ordering::Relaxed)
        }
    }

    impl ComponentRepositoryPort for MockRepo {
        type Error = String;
        type Tx<'tx> = MockTx;

        fn find(
            &self,
            _tx: &mut Self::Tx<'_>,
            _id: &ResourceId,
        ) -> impl Future<Output = Result<Option<Component>, Self::Error>> + Send {
            self.find_calls.fetch_add(1, Ordering::Relaxed);
            ready(Ok(self.component.lock().unwrap().clone()))
        }

        fn upsert_component(
            &self,
            _tx: &mut Self::Tx<'_>,
            _component: &Component,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.upsert_calls.fetch_add(1, Ordering::Relaxed);
            ready(Ok(()))
        }

        fn delete_component(
            &self,
            _tx: &mut Self::Tx<'_>,
            _component_id: &ResourceId,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.delete_calls.fetch_add(1, Ordering::Relaxed);
            *self.component.lock().unwrap() = None;
            ready(Ok(()))
        }
    }

    #[derive(Clone, Default)]
    struct MockOutboxWriter {
        append_calls: Arc<AtomicUsize>,
        view_ids: Arc<Mutex<Vec<String>>>,
    }

    impl MockOutboxWriter {
        fn append_call_count(&self) -> usize {
            self.append_calls.load(Ordering::Relaxed)
        }

        fn view_ids(&self) -> Vec<String> {
            self.view_ids.lock().unwrap().clone()
        }
    }

    impl OutboxWriter for MockOutboxWriter {
        type Error = String;
        type Tx<'tx> = MockTx;

        fn append(
            &self,
            _tx: &mut Self::Tx<'_>,
            envelope: EventEnvelope,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.append_calls.fetch_add(1, Ordering::Relaxed);
            self.view_ids.lock().unwrap().push(envelope.view_id);
            ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn create_demo_component_uses_core_component_lifecycle_uow() {
        let uow = MockUow::default();
        let repo = MockRepo::default();
        let writer = MockOutboxWriter::default();
        let publisher =
            DomainEventOutboxPublisher::new(VIEW_ID, ComponentDomainEventMapper, writer.clone());

        let lifecycle = ComponentLifecycleUow::new(uow.clone(), repo.clone(), publisher);
        let service = DemoComponentService::new(lifecycle);

        service
            .create_demo_component(CreateDemoComponentRequest {
                id: "demo-component-1".to_string(),
                name: "Demo component".to_string(),
                description: Some("demo".to_string()),
                parent: None,
                children: vec![],
                parameters: Default::default(),
            })
            .await
            .unwrap();

        assert_eq!(uow.execute_call_count(), 1);
        assert_eq!(repo.upsert_call_count(), 1);
        assert_eq!(writer.append_call_count(), 1);
        assert_eq!(writer.view_ids(), vec![VIEW_ID.to_string()]);
    }

    #[tokio::test]
    async fn delete_demo_component_uses_core_component_lifecycle_uow() {
        let uow = MockUow::default();
        let format = IdFormat::new(None, None, None).unwrap();
        let repo = MockRepo::with_component(
            Component::create(
                ResourceId::new("demo-component-1".to_string(), format).unwrap(),
                "Demo component".to_string(),
                Some("demo".to_string()),
                None,
                vec![],
                Default::default(),
            )
            .unwrap()
            .0,
        );
        let writer = MockOutboxWriter::default();
        let publisher =
            DomainEventOutboxPublisher::new(VIEW_ID, ComponentDomainEventMapper, writer.clone());

        let lifecycle = ComponentLifecycleUow::new(uow.clone(), repo.clone(), publisher);
        let service = DemoComponentService::new(lifecycle);

        service
            .delete_demo_component(DeleteDemoComponentRequest {
                id: "demo-component-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(uow.execute_call_count(), 1);
        assert_eq!(repo.find_call_count(), 1);
        assert_eq!(repo.delete_call_count(), 1);
        assert_eq!(writer.append_call_count(), 1);
        assert_eq!(writer.view_ids(), vec![VIEW_ID.to_string()]);
    }
}
