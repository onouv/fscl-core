use std::future::Future;
use std::pin::Pin;

use sqlx::Error;
use super::{DatabasePort, SqlxPgDatabase};
use crate::core::ports::UnitOfWorkPort;



/// Unit of Work pattern implementation for a database connection, allowing to bundle multiple operations 
/// on repositories in a single transaction.
/// 
/// The `execute` method takes a closure that performs operations within a transaction.
#[derive(Clone)]
pub struct UnitOfWork<D>
where
    D: DatabasePort,
{
    db: D,
}

impl<D> UnitOfWork<D>
where
    D: DatabasePort,
{
    pub fn new(db: D) -> Self {
        Self { db }
    }

    pub async fn execute<T, F>(&self, operation: F) -> Result<T, D::Error>
    where
        T: Send,
        F: for<'c> FnOnce(
                &'c mut D::Tx<'c>,
            )
                -> Pin<Box<dyn Future<Output = Result<T, D::Error>> + Send + 'c>>
            + Send
            + 'static,
    {
        self.db.execute_in_transaction(operation).await
    }
}



impl UnitOfWork<SqlxPgDatabase> {
    // Convenience constructor for production composition in main.
    pub async fn connect(connection_string: &str) -> Result<Self, Error> {
        Ok(Self::new(SqlxPgDatabase::connect(connection_string).await?))
    }
}

impl<D> UnitOfWorkPort for UnitOfWork<D>
where
    D: DatabasePort,
{
    type Error = D::Error;
    type Tx<'tx> = D::Tx<'tx>;

    fn execute<T, F>(&self, operation: F) -> impl Future<Output = Result<T, Self::Error>> + Send + '_
    where
        T: Send,
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        async move { self.db.execute_in_transaction(operation).await }
    }
}
