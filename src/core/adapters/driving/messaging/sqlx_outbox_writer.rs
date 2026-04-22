use std::future::Future;

use fscl_messaging::EventEnvelope;
use sqlx::{Error, Postgres, pool::PoolConnection};

use super::OutboxWriter;

#[derive(Clone, Default)]
pub struct SqlxOutboxWriter;

impl OutboxWriter for SqlxOutboxWriter {
    type Error = Error;
    type Tx<'tx> = PoolConnection<Postgres>;

    fn append(
        &self,
        tx: &mut Self::Tx<'_>,
        envelope: EventEnvelope,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            sqlx::query(
                "INSERT INTO outbox (id, occurred_at, event_type, aggregate_type, aggregate_id, view_id, payload) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(envelope.id)
            .bind(envelope.occurred_at)
            .bind(&envelope.event_type)
            .bind(envelope.aggregate_type.as_str())
            .bind(&envelope.aggregate_id)
            .bind(&envelope.view_id)
            .bind(&envelope.payload)
            .execute(&mut **tx)
            .await?;

            Ok(())
        }
    }
}
