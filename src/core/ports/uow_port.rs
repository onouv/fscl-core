use std::future::Future;
use std::pin::Pin;

/// Port for Unit of Work pattern, abstracting transaction management and execution of operations 
/// within a transaction. Implementations of this trait may execute a series of operations, 
/// but must do so within the transaction, ensuring that all operations either succeed or fail together.
pub trait UnitOfWorkPort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn execute<T, F>(&self, operation: F) -> impl Future<Output = Result<T, Self::Error>> + Send + '_
    where
        T: Send,
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'tx>>
            + Send
            + 'static;
}
