use super::DatabasePort;
use sqlx::{PgPool, Postgres, Error};
use std::future::Future;
use std::pin::Pin;

/// Adapter for a postgres database connection pool. 
#[derive(Clone)]
pub struct SqlxPgDatabase {
    pool: PgPool,
}

impl SqlxPgDatabase {
    // note: PgPool is Clone and the clone remains tied to same connection pool.
    pub async fn connect(connection_string: &str) -> Result<Self, Error> {
        let pool = PgPool::connect(connection_string).await?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DatabasePort for SqlxPgDatabase {
    type Error = Error;
    type Tx<'tx> = sqlx::pool::PoolConnection<Postgres>;

    #[allow(clippy::manual_async_fn)] // since we need to apply trait bounds on future.
    fn execute_in_transaction<T, F>(&self, operation: F) -> impl Future<Output = Result<T, Self::Error>> + Send + '_
    where
        T: Send,
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        async move {
            let mut conn = self.pool.acquire().await?;
            sqlx::query("BEGIN").execute(&mut *conn).await?;

            let result = operation(&mut conn).await;

            match result {
                Ok(res) => {
                    sqlx::query("COMMIT").execute(&mut *conn).await?;
                    Ok(res)
                }
                Err(e) => {
                    sqlx::query("ROLLBACK").execute(&mut *conn).await?;
                    Err(e)
                }
            }
        }
    }
}