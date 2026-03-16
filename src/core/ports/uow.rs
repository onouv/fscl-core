use std::future::Future;
use std::pin::Pin;

use sqlx::{Database, Transaction};

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
