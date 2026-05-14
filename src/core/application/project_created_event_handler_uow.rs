use std::future::Future;

use fscl_messaging::ProjectCreatedEvent;

use crate::core::application::{
    InitializeProjectIdFormatError, InitializeProjectIdFormatRequest, ProjectIdFormatInitializerUow,
};
use crate::core::domain::{ProjectId, ProjectIdError, ResourceIdError};
use crate::core::ports::{ProjectIdFormatRepositoryPort, UnitOfWorkPort};

#[derive(Debug, thiserror::Error)]
pub enum HandleProjectCreatedEventError<E> {
    #[error(transparent)]
    InvalidProjectId(#[from] ProjectIdError),
    #[error(transparent)]
    InvalidFormat(#[from] ResourceIdError),
    #[error("Infrastructure error during project-created handling")]
    Infrastructure(E),
}

#[derive(Clone)]
pub struct ProjectCreatedEventHandlerUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectIdFormatRepositoryPort<Error = U::Error>,
{
    initializer: ProjectIdFormatInitializerUow<U, R>,
}

impl<U, R> ProjectCreatedEventHandlerUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectIdFormatRepositoryPort<Error = U::Error> + 'static,
{
    pub fn new(initializer: ProjectIdFormatInitializerUow<U, R>) -> Self {
        Self { initializer }
    }

    pub fn handle(
        &self,
        event: ProjectCreatedEvent,
    ) -> impl Future<Output = Result<(), HandleProjectCreatedEventError<U::Error>>> + Send
    where
        R: for<'tx> ProjectIdFormatRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        let initializer = self.initializer.clone();

        async move {
            let project_id = ProjectId::new(event.project_id)?;

            match initializer
                .initialize(InitializeProjectIdFormatRequest {
                    project_id,
                    prefix: event.prefix,
                    separator: event.separator,
                    block_length: event.block_length,
                })
                .await
            {
                Ok(_) => Ok(()),
                Err(InitializeProjectIdFormatError::AlreadyInitialized) => Ok(()),
                Err(InitializeProjectIdFormatError::InvalidFormat(error)) => {
                    Err(HandleProjectCreatedEventError::InvalidFormat(error))
                }
                Err(InitializeProjectIdFormatError::EmptyProjectId) => Err(
                    HandleProjectCreatedEventError::InvalidProjectId(ProjectIdError::Empty),
                ),
                Err(InitializeProjectIdFormatError::Infrastructure(error)) => {
                    Err(HandleProjectCreatedEventError::Infrastructure(error))
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
    use crate::core::application::ProjectIdFormatInitializerUow;
    use crate::core::domain::IdFormat;
    use crate::core::ports::{ProjectIdFormatRepositoryPort, UnitOfWorkPort};

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
        formats: Arc<Mutex<HashMap<String, IdFormat>>>,
    }

    impl MockRepo {
        fn count(&self) -> usize {
            self.formats.lock().unwrap().len()
        }
    }

    impl ProjectIdFormatRepositoryPort for MockRepo {
        type Error = String;
        type Tx<'tx> = MockTx;

        fn find_project_id_format(
            &self,
            _tx: &mut Self::Tx<'_>,
            project_id: &str,
        ) -> impl Future<Output = Result<Option<IdFormat>, Self::Error>> + Send {
            ready(Ok(self.formats.lock().unwrap().get(project_id).cloned()))
        }

        fn save_project_id_format(
            &self,
            _tx: &mut Self::Tx<'_>,
            project_id: &str,
            format: &IdFormat,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.formats
                .lock()
                .unwrap()
                .insert(project_id.to_string(), format.clone());
            ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn handles_project_created_event_idempotently() {
        let repo = MockRepo::default();
        let initializer = ProjectIdFormatInitializerUow::new(MockUow, repo.clone());
        let handler = ProjectCreatedEventHandlerUow::new(initializer);
        let event = ProjectCreatedEvent {
            event_id: Uuid::new_v4(),
            occurred_at: chrono::Utc::now(),
            project_id: "project-a".to_string(),
            prefix: Some("=".to_string()),
            separator: Some(".".to_string()),
            block_length: Some(4),
        };

        handler.handle(event.clone()).await.unwrap();
        handler.handle(event).await.unwrap();

        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn rejects_empty_project_id() {
        let repo = MockRepo::default();
        let initializer = ProjectIdFormatInitializerUow::new(MockUow, repo);
        let handler = ProjectCreatedEventHandlerUow::new(initializer);
        let event = ProjectCreatedEvent {
            event_id: Uuid::new_v4(),
            occurred_at: chrono::Utc::now(),
            project_id: String::new(),
            prefix: Some("=".to_string()),
            separator: Some(".".to_string()),
            block_length: Some(4),
        };

        let result = handler.handle(event).await;

        assert!(matches!(
            result,
            Err(HandleProjectCreatedEventError::InvalidProjectId(_))
        ));
    }
}
