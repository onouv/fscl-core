use std::future::Future;
use std::pin::Pin;

use crate::core::domain::{Resource, ResourceId};

pub trait ResourceLifecycleWorkflowPort: Clone + Send + Sync {
    type Error: Send;
    type CreateTx<'tx>: Send;
    type DeleteTx<'tx>: Send;

    fn create_resource<R>(&self, resource: &R) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        R: Resource + Send + Sync;

    fn create_resource_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn create_resource_with<F>(&self, resource_id: ResourceId, client_algorithm: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::CreateTx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static;

    fn delete_resource<R>(&self, resource: &R) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        R: Resource + Send + Sync;

    fn delete_resource_by_id(&self, resource_id: ResourceId) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn delete_resource_with<F>(&self, resource_id: ResourceId, client_algorithm: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::DeleteTx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static;
}