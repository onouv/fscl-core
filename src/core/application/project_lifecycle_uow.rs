use std::future::Future;

use crate::core::domain::{IdFormat, Project, ProjectError, ProjectId, ResourceIdError};
use crate::core::ports::{ProjectRepositoryPort, UnitOfWorkPort};

#[derive(Debug, Clone)]
pub struct CreateProjectRequest {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub prefix: Option<String>,
    pub separator: Option<String>,
    pub block_length: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectError<E> {
    #[error(transparent)]
    InvalidName(#[from] ProjectError),
    #[error(transparent)]
    InvalidFormat(#[from] ResourceIdError),
    #[error("Project already exists")]
    AlreadyExists,
    #[error("Infrastructure error during project creation")]
    Infrastructure(E),
}

#[derive(Clone)]
pub struct ProjectLifecycleUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectRepositoryPort<Error = U::Error>,
{
    unit_of_work: U,
    repository: R,
}

impl<U, R> ProjectLifecycleUow<U, R>
where
    U: UnitOfWorkPort,
    R: ProjectRepositoryPort<Error = U::Error> + 'static,
{
    pub fn new(unit_of_work: U, repository: R) -> Self {
        Self {
            unit_of_work,
            repository,
        }
    }

    pub fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> impl Future<Output = Result<Project, CreateProjectError<U::Error>>> + Send
    where
        R: for<'tx> ProjectRepositoryPort<Error = U::Error, Tx<'tx> = U::Tx<'tx>>,
    {
        let unit_of_work = self.unit_of_work.clone();
        let repository = self.repository.clone();

        async move {
            let id_format = IdFormat::new(request.prefix, request.separator, request.block_length)?;
            let project = Project::new(request.id, request.name, request.description, id_format)?;
            let project_to_return = project.clone();

            let saved = unit_of_work
                .execute(move |tx| {
                    let repository = repository.clone();
                    let project = project.clone();
                    Box::pin(async move {
                        if repository.find_project(tx, &project.id).await?.is_some() {
                            return Ok(false);
                        }
                        repository.save_project(tx, &project).await?;
                        Ok(true)
                    })
                })
                .await
                .map_err(CreateProjectError::Infrastructure)?;

            if !saved {
                return Err(CreateProjectError::AlreadyExists);
            }

            Ok(project_to_return)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::{Future, ready};
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    use super::{CreateProjectError, CreateProjectRequest, ProjectLifecycleUow};
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

    fn project_id() -> ProjectId {
        ProjectId::new("project-a".to_string()).unwrap()
    }

    #[tokio::test]
    async fn create_project_saves_project() {
        let repo = MockRepo::default();
        let uow = ProjectLifecycleUow::new(MockUow, repo.clone());

        uow.create_project(CreateProjectRequest {
            id: project_id(),
            name: "Project A".to_string(),
            description: Some("desc".to_string()),
            prefix: Some("=".to_string()),
            separator: Some(".".to_string()),
            block_length: Some(4),
        })
        .await
        .unwrap();

        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn create_project_returns_already_exists_on_duplicate() {
        let repo = MockRepo::default();
        let uow = ProjectLifecycleUow::new(MockUow, repo.clone());

        let request = || CreateProjectRequest {
            id: project_id(),
            name: "Project A".to_string(),
            description: None,
            prefix: None,
            separator: None,
            block_length: None,
        };

        uow.create_project(request()).await.unwrap();
        let result = uow.create_project(request()).await;

        assert!(matches!(result, Err(CreateProjectError::AlreadyExists)));
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn create_project_rejects_empty_name() {
        let repo = MockRepo::default();
        let uow = ProjectLifecycleUow::new(MockUow, repo);

        let result = uow
            .create_project(CreateProjectRequest {
                id: project_id(),
                name: String::new(),
                description: None,
                prefix: None,
                separator: None,
                block_length: None,
            })
            .await;

        assert!(matches!(result, Err(CreateProjectError::InvalidName(_))));
    }
}
