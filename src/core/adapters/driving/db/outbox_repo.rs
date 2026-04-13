use sqlx::{Error, Transaction, postgres::Postgres};
use crate::adapters::driving::messaging::event_message::EventMessage;


#[derive(Clone)]
pub struct OutboxRepo {}

impl OutboxRepo {
    pub async fn save(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &EventMessage,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO outbox (id, occurred_at, event_type, aggregate_type, aggregate_id, view_id, payload) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
            .bind(event.id)
            .bind(event.occurred_at)
            .bind(&event.event_type)
            .bind(event.aggregate_type.as_str())
            .bind(&event.aggregate_id)
            .bind(&event.view_id)
            .bind(&event.payload)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}