use std::future::Future;

use crate::core::domain::IdFormat;

pub trait ProjectIdFormatRepositoryPort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn find_project_id_format(
        &self,
        tx: &mut Self::Tx<'_>,
        project_id: &str,
    ) -> impl Future<Output = Result<Option<IdFormat>, Self::Error>> + Send;

    fn save_project_id_format(
        &self,
        tx: &mut Self::Tx<'_>,
        project_id: &str,
        format: &IdFormat,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
