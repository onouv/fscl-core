use std::pin::Pin;

use sqlx::{Error, Postgres, Transaction, PgPool};

use crate::core::ports::UnitOfWorkPort;

#[derive(Clone)]
pub struct UnitOfWork {
    pool: PgPool,
}

impl UnitOfWork {
    // note: PgPool is Clone and the clone remains tied to same connection pool
    pub async fn new() -> Result<Self, Error> {
        let pool = PgPool::connect("postgres://postgres:postgres@localhost/process").await?;

        Ok(Self { pool })
    }

    pub async fn execute<F>(&self, operation: F) -> Result<(), Error>
    where
        F: for<'c> FnOnce(
                &'c mut Transaction<'_, Postgres>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'c>>
            + Send,
    {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        // Execute the operation within the transaction
        let result = operation(&mut tx).await;

        match result {
            Ok(res) => {
                tx.commit().await?;
                Ok(res)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }
}

impl UnitOfWorkPort for UnitOfWork {
    type Db = Postgres;
    type Error = Error;

    fn execute<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        F: for<'tx> FnOnce(
                &'tx mut Transaction<'_, Self::Db>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send,
    {
        Self::execute(self, operation)
    }
}
