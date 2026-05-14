use std::future::Future;

use crate::core::domain::{Project, ProjectId};

pub trait ProjectRepositoryPort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn find_project(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &ProjectId,
    ) -> impl Future<Output = Result<Option<Project>, Self::Error>> + Send;

    fn save_project(
        &self,
        tx: &mut Self::Tx<'_>,
        project: &Project,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
