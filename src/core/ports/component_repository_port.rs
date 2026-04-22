use std::future::Future;

use crate::core::domain::{Component, ResourceId};

pub trait ComponentRepositoryPort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn find(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &ResourceId,
    ) -> impl Future<Output = Result<Option<Component>, Self::Error>> + Send;

    fn upsert_component(
        &self,
        tx: &mut Self::Tx<'_>,
        component: &Component,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn delete_component(
        &self,
        tx: &mut Self::Tx<'_>,
        component_id: &ResourceId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
