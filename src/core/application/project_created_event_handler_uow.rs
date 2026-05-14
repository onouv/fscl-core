use std::future::Future;

use fscl_messaging::ProjectCreatedEvent;

use crate::core::application::{CreateProjectError, CreateProjectRequest, ProjectLifecycleUow};
use crate::core::domain::{ProjectError, ProjectId, ProjectIdError, ResourceIdError};
use crate::core::ports::{ProjectRepositoryPort, UnitOfWorkPort};

#[derive(Debug, thiserror::Error)]
pub enum HandleProjectCreatedEventError<E> {
    #[error(transparent)]
    InvalidProjectId(#[from] ProjectIdError),
    #[error(transparent)]
    InvalidName(ProjectError),
    #[error(transparent)]
    InvalidFormat(#[from] ResourceIdError),
    #[error("Infrastructure error during project-created handling")]
    Infrastructure(E),
}

#[derive(Clone)]
pub struct ProjectCreatedEventHandlerUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectRepositoryPort<Error = U::Error>,
{
    lifecycle: ProjectLifecycleUow<U, R>,
}

impl<U, R> ProjectCreatedEventHandlerUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectRepositoryPort<Error = U::Error> + 'static,
{
    pub fn new(lifecycle: ProjectLifecycleUow<U, R>) -> Self {
        Self { lifecycle }
    }

    pub fn handle(
        &self,
        event: ProjectCreatedEvent,
    ) -> impl Future<Output = Result<(), HandleProjectCreatedEventError<U::Error>>> + Send
    where
        R: for<'tx> ProjectRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        let lifecycle = self.lifecycle.clone();

        async move {
            let project_id = ProjectId::new(event.project_id)?;

            match lifecycle
                .create_project(CreateProjectRequest {
                    id: project_id,
                    name: event.name,
                    description: event.description,
                    prefix: event.prefix,
                    separator: event.separator,
                    block_length: event.block_length,
                })
                .await
            {
                Ok(_) => Ok(()),
                Err(CreateProjectError::AlreadyExists) => Ok(()),
                Err(CreateProjectError::InvalidName(e)) => {
                    Err(HandleProjectCreatedEventError::InvalidName(e))
                }
                Err(CreateProjectError::InvalidFormat(e)) => {
                    Err(HandleProjectCreatedEventError::InvalidFormat(e))
                }
                Err(CreateProjectError::Infrastructure(e)) => {
                    Err(HandleProjectCreatedEventError::Infrastructure(e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::{Future, ready};
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    use fscl_messaging::ProjectCreatedEvent;
    use uuid::Uuid;

    use super::{HandleProjectCreatedEventError, ProjectCreatedEventHandlerUow};
    use crate::core::application::ProjectLifecycleUow;
    use crate::core::domain::{Project, ProjectId};
    use crate::core::ports::{ProjectRepositoryPort, UnitOfWorkPort};

    #[derive(Clone, Default)]
    struct MockTx;

    #[derive(Clone, Default)]
    struct MockUow;

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
                let mut tx = MockTx;
                operation(&mut tx).await
            }
        }
    }

    #[derive(Clone, Default)]
    struct MockRepo {
        projects: Arc<Mutex<HashMap<ProjectId, Project>>>,
    }

    impl MockRepo {
        fn count(&self) -> usize {
            self.projects.lock().unwrap().len()
        }
    }

    impl ProjectRepositoryPort for MockRepo {
        type Error = String;
        type Tx<'tx> = MockTx;

        fn find_project(
            &self,
            _tx: &mut Self::Tx<'_>,
            id: &ProjectId,
        ) -> impl Future<Output = Result<Option<Project>, Self::Error>> + Send {
            ready(Ok(self.projects.lock().unwrap().get(id).cloned()))
        }

        fn save_project(
            &self,
            _tx: &mut Self::Tx<'_>,
            project: &Project,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.projects
                .lock()
                .unwrap()
                .insert(project.id.clone(), project.clone());
            ready(Ok(()))
        }
    }

    fn event(project_id: &str, name: &str) -> ProjectCreatedEvent {
        ProjectCreatedEvent {
            event_id: Uuid::new_v4(),
            occurred_at: chrono::Utc::now(),
            project_id: project_id.to_string(),
            name: name.to_string(),
            description: None,
            prefix: Some("=".to_string()),
            separator: Some(".".to_string()),
            block_length: Some(4),
        }
    }

    #[tokio::test]
    async fn handles_project_created_event_idempotently() {
        let repo = MockRepo::default();
        let handler =
            ProjectCreatedEventHandlerUow::new(ProjectLifecycleUow::new(MockUow, repo.clone()));
        let e = event("project-a", "Project A");

        handler.handle(e.clone()).await.unwrap();
        handler.handle(e).await.unwrap();

        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn rejects_empty_project_id() {
        let repo = MockRepo::default();
        let handler = ProjectCreatedEventHandlerUow::new(ProjectLifecycleUow::new(MockUow, repo));

        let result = handler.handle(event("", "Project A")).await;

        assert!(matches!(
            result,
            Err(HandleProjectCreatedEventError::InvalidProjectId(_))
        ));
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let repo = MockRepo::default();
        let handler = ProjectCreatedEventHandlerUow::new(ProjectLifecycleUow::new(MockUow, repo));

        let result = handler.handle(event("project-a", "")).await;

        assert!(matches!(
            result,
            Err(HandleProjectCreatedEventError::InvalidName(_))
        ));
    }
}
