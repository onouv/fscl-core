use std::future::Future;
use std::pin::Pin;

use sqlx::{Database, Transaction};

/// Port for Unit of Work pattern, abstracting transaction management and execution of operations 
/// within a transaction. Implementations of this trait must executing a series of 
/// operations within the transaction, ensuring that all operations either succeed or fail together. 
/// The trait also serves to isolate the application layer from the actual database technology.
pub trait UnitOfWorkPort: Clone + Send + Sync {
    type Db: Database;
    type Error: Send;

    fn execute<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Transaction<'_, Self::Db>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send;
}
