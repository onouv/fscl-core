use std::future::Future;

use crate::core::domain::{IdFormat, ProjectId, ResourceIdError};
use crate::core::ports::{ProjectIdFormatRepositoryPort, UnitOfWorkPort};

#[derive(Debug, Clone)]
pub struct InitializeProjectIdFormatRequest {
    pub project_id: ProjectId,
    pub prefix: Option<String>,
    pub separator: Option<String>,
    pub block_length: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum InitializeProjectIdFormatError<E> {
    #[error("Project ID cannot be empty")]
    EmptyProjectId,
    #[error(transparent)]
    InvalidFormat(#[from] ResourceIdError),
    #[error("Project ID format is already initialized")]
    AlreadyInitialized,
    #[error("Infrastructure error during project ID format initialization")]
    Infrastructure(E),
}

#[derive(Clone)]
pub struct ProjectIdFormatInitializerUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectIdFormatRepositoryPort<Error = U::Error>,
{
    unit_of_work: U,
    repository: R,
}

impl<U, R> ProjectIdFormatInitializerUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectIdFormatRepositoryPort<Error = U::Error> + 'static,
{
    pub fn new(unit_of_work: U, repository: R) -> Self {
        Self {
            unit_of_work,
            repository,
        }
    }

    pub fn initialize(
        &self,
        request: InitializeProjectIdFormatRequest,
    ) -> impl Future<Output = Result<IdFormat, InitializeProjectIdFormatError<U::Error>>> + Send
    where
        R: for<'tx> ProjectIdFormatRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        let unit_of_work = self.unit_of_work.clone();
        let repository = self.repository.clone();

        async move {
            let project_id_str = request.project_id.as_str().to_string();
            let format = IdFormat::new(request.prefix, request.separator, request.block_length)?;
            let format_for_save = format.clone();

            let created = unit_of_work
                .execute(move |tx| {
                    let project_id_str = project_id_str.clone();
                    Box::pin(async move {
                        if repository.find_project_id_format(tx, &project_id_str).await?.is_some() {
                            return Ok(false);
                        }

                        repository
                            .save_project_id_format(tx, &project_id_str, &format_for_save)
                            .await?;

                        Ok(true)
                    })
                })
                .await
                .map_err(InitializeProjectIdFormatError::Infrastructure)?;

            if !created {
                return Err(InitializeProjectIdFormatError::AlreadyInitialized);
            }

            Ok(format)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::{Future, ready};
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    use super::{
        InitializeProjectIdFormatError, InitializeProjectIdFormatRequest, ProjectIdFormatInitializerUow,
    };
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
                ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'tx>>
                + Send
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
    async fn initializes_and_saves_project_format() {
        let service = ProjectIdFormatInitializerUow::new(MockUow, MockRepo::default());

        let result = service
            .initialize(InitializeProjectIdFormatRequest {
                project_id: "project-a".to_string(),
                prefix: Some("=".to_string()),
                separator: Some(".".to_string()),
                block_length: Some(4),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn rejects_reinitialization_of_existing_project_format() {
        let repo = MockRepo::default();
        let service = ProjectIdFormatInitializerUow::new(MockUow, repo.clone());

        let first = service
            .initialize(InitializeProjectIdFormatRequest {
                project_id: "project-a".to_string(),
                prefix: Some("=".to_string()),
                separator: Some(".".to_string()),
                block_length: Some(4),
            })
            .await;
        assert!(first.is_ok());

        let second = service
            .initialize(InitializeProjectIdFormatRequest {
                project_id: "project-a".to_string(),
                prefix: Some("#".to_string()),
                separator: Some("-".to_string()),
                block_length: Some(4),
            })
            .await;

        assert!(matches!(
            second,
            Err(InitializeProjectIdFormatError::AlreadyInitialized)
        ));
    }
}
