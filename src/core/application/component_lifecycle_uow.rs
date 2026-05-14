use std::collections::BTreeMap;
use std::future::Future;

use crate::core::domain::{
    Component, ComponentError, IdFormat, ProjectId, ResourceId, ResourceIdError,
};
use crate::core::ports::{ComponentRepositoryPort, DomainEventPublisherPort, UnitOfWorkPort};

#[derive(Debug, Clone)]
pub struct CreateComponentRequest {
    pub project_id: ProjectId,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DeleteComponentRequest {
    pub project_id: ProjectId,
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateComponentError<E> {
    #[error(transparent)]
    InvalidId(#[from] ResourceIdError),
    #[error(transparent)]
    Domain(#[from] ComponentError),
    #[error("Infrastructure error during create component transaction")]
    Infrastructure(E),
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteComponentError<E> {
    #[error(transparent)]
    InvalidId(#[from] ResourceIdError),
    #[error("Component not found")]
    NotFound,
    #[error("Infrastructure error during delete component transaction")]
    Infrastructure(E),
    #[error("Domain logic did not allow component deletion")]
    Domain(#[from] ComponentError),
}

enum DeleteComponentOutcome {
    Deleted,
    NotFound,
    DomainError(ComponentError),
}

#[derive(Clone)]
pub struct ComponentLifecycleUow<U, R, P>
where
    U: UnitOfWorkPort,
    R: ComponentRepositoryPort<Error = U::Error>,
    P: DomainEventPublisherPort<Error = U::Error>,
{
    unit_of_work: U,
    repository: R,
    publisher: P,
}

impl<U, R, S> ComponentLifecycleUow<U, R, S>
where
    U: UnitOfWorkPort,
    R: ComponentRepositoryPort<Error = U::Error> + 'static,
    S: DomainEventPublisherPort<Error = U::Error> + 'static,
{
    pub fn new(unit_of_work: U, repository: R, publisher: S) -> Self {
        Self {
            unit_of_work,
            repository,
            publisher,
        }
    }

    pub fn create_component(
        &self,
        request: CreateComponentRequest,
    ) -> impl Future<Output = Result<(), CreateComponentError<U::Error>>> + Send
    where
        R: for<'tx> ComponentRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
        S: for<'tx> DomainEventPublisherPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        let unit_of_work = self.unit_of_work.clone();
        let repository = self.repository.clone();
        let publisher = self.publisher.clone();
        let format = IdFormat::new(None, None, None).unwrap();
        async move {
            let project_id = request.project_id.clone();
            let id = ResourceId::new(project_id.clone(), request.id, format.clone())?;
            let parent = request
                .parent
                .map(|parent| ResourceId::new(project_id.clone(), parent, format.clone()))
                .transpose()
                .map_err(CreateComponentError::InvalidId)?;

            let mut children = Vec::with_capacity(request.children.len());
            for child in request.children {
                children.push(ResourceId::new(project_id.clone(), child, format.clone())?);
            }

            let (component, event) = Component::create(
                id,
                request.name,
                request.description,
                parent,
                children,
                request.parameters,
            )?;

            unit_of_work
                .execute(move |tx| {
                    Box::pin(async move {
                        repository.upsert_component(tx, &component).await?;
                        publisher.publish(tx, &event).await?;
                        Ok(())
                    })
                })
                .await
                .map_err(CreateComponentError::Infrastructure)
        }
    }

    pub fn delete_component(
        &self,
        request: DeleteComponentRequest,
    ) -> impl Future<Output = Result<(), DeleteComponentError<U::Error>>> + Send
    where
        R: for<'tx> ComponentRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
        S: for<'tx> DomainEventPublisherPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        let unit_of_work = self.unit_of_work.clone();
        let repository = self.repository.clone();
        let publisher = self.publisher.clone();
        let format = IdFormat::new(None, None, None).unwrap();

        async move {
            let project_id = request.project_id.clone();
            let id = ResourceId::new(project_id, request.id, format)?;

            let outcome = unit_of_work
                .execute(move |tx| {
                    let repository = repository.clone();
                    let publisher = publisher.clone();
                    let id = id.clone();

                    Box::pin(async move {
                        let component = match repository.find(tx, &id).await? {
                            Some(component) => component,
                            None => return Ok(DeleteComponentOutcome::NotFound),
                        };

                        match component.delete() {
                            Ok(events) => {
                                repository.delete_component(tx, &id).await?;

                                for event in events {
                                    publisher.publish(tx, &event).await?;
                                }

                                Ok(DeleteComponentOutcome::Deleted)
                            }
                            Err(error) => Ok(DeleteComponentOutcome::DomainError(error)),
                        }
                    })
                })
                .await
                .map_err(DeleteComponentError::Infrastructure)?;

            match outcome {
                DeleteComponentOutcome::Deleted => Ok(()),
                DeleteComponentOutcome::NotFound => Err(DeleteComponentError::NotFound),
                DeleteComponentOutcome::DomainError(error) => {
                    Err(DeleteComponentError::Domain(error))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::{Future, ready};
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{ComponentLifecycleUow, CreateComponentRequest, DeleteComponentRequest};
    use crate::core::domain::{Component, DomainEvent, IdFormat, ProjectId, ResourceId};
    use crate::core::ports::{ComponentRepositoryPort, DomainEventPublisherPort, UnitOfWorkPort};

    // static VIEW_ID: &str = "test-view";

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
        upsert_calls: Arc<AtomicUsize>,
        delete_calls: Arc<AtomicUsize>,
    }

    impl MockRepo {
        fn with_component(component: Component) -> Self {
            Self {
                component: Arc::new(Mutex::new(Some(component))),
                upsert_calls: Arc::new(AtomicUsize::new(0)),
                delete_calls: Arc::new(AtomicUsize::new(0)),
            }
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
    struct MockEventSink {
        append_calls: Arc<AtomicUsize>,
    }

    impl MockEventSink {
        fn append_call_count(&self) -> usize {
            self.append_calls.load(Ordering::Relaxed)
        }
    }

    impl DomainEventPublisherPort for MockEventSink {
        type Error = String;
        type Tx<'tx> = MockTx;

        fn publish(
            &self,
            _tx: &mut Self::Tx<'_>,
            _event: &DomainEvent,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.append_calls.fetch_add(1, Ordering::Relaxed);
            ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn create_component_runs_repo_and_event_sink_inside_uow() {
        let uow = MockUow::default();
        let repo = MockRepo::default();
        let sink = MockEventSink::default();
        let project_id = ProjectId::new("project-a".to_string()).unwrap();

        let service = ComponentLifecycleUow::new(uow.clone(), repo.clone(), sink.clone());

        service
            .create_component(CreateComponentRequest {
                project_id,
                id: "component-1".to_string(),
                name: "Main component".to_string(),
                description: Some("demo".to_string()),
                parent: None,
                children: vec![],
                parameters: Default::default(),
            })
            .await
            .unwrap();

        assert_eq!(uow.execute_call_count(), 1);
        assert_eq!(repo.upsert_call_count(), 1);
        assert_eq!(sink.append_call_count(), 1);
    }

    #[tokio::test]
    async fn delete_component_runs_repo_and_event_sink_inside_uow() {
        let uow = MockUow::default();
        let project_id = ProjectId::new("project-a".to_string()).unwrap();
        let format = IdFormat::new(None, None, None).unwrap();
        let (component, _) = Component::create(
            ResourceId::new(project_id.clone(), "component-1".to_string(), format).unwrap(),
            "Main component".to_string(),
            Some("demo".to_string()),
            None,
            vec![],
            Default::default(),
        )
        .unwrap();
        let repo = MockRepo::with_component(component);
        let sink = MockEventSink::default();

        let service = ComponentLifecycleUow::new(uow.clone(), repo.clone(), sink.clone());

        service
            .delete_component(DeleteComponentRequest {
                project_id,
                id: "component-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(uow.execute_call_count(), 1);
        assert_eq!(repo.delete_call_count(), 1);
        assert_eq!(sink.append_call_count(), 1);
    }

    #[tokio::test]
    async fn delete_component_returns_not_found_when_repo_has_no_component() {
        let uow = MockUow::default();
        let repo = MockRepo::default();
        let sink = MockEventSink::default();
        let project_id = ProjectId::new("project-a".to_string()).unwrap();

        let service = ComponentLifecycleUow::new(uow.clone(), repo.clone(), sink.clone());

        let result = service
            .delete_component(DeleteComponentRequest {
                project_id,
                id: "component-1".to_string(),
            })
            .await;

        assert!(matches!(result, Err(super::DeleteComponentError::NotFound)));
        assert_eq!(uow.execute_call_count(), 1);
        assert_eq!(repo.delete_call_count(), 0);
        assert_eq!(sink.append_call_count(), 0);
    }
}
