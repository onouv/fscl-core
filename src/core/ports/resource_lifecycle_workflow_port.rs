use std::future::Future;

use crate::{ResourceId, core::domain::Resource};

/// This workflow trait bundles unit of work implementations related to 
/// the lifecycle of FSCL resources, such as creation and deletion. 
/// It allows clients to perform these operations without needing to know 
/// the details of the underlying unit of work implementations.
///  
/// This design prevents an explosion of generic parameters for UoW by
/// seperating them into different workflows.
pub trait ResourceLifecycleWorkflowPort: Clone + Send + Sync {
    type Error: Send;
    type CreateTx<'tx>: Send;
    type DeleteTx<'tx>: Send;

    /// Create a resource of type R with the given resource data.
    fn create_resource<R>(&self, resource: &R) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        R: Resource + Send + Sync;

    fn delete_resource(&self, resource_id: &ResourceId) -> impl Future<Output = Result<(), Self::Error>> + Send;
    
}