use sqlx::{Error, Transaction, postgres::Postgres};
use crate::adapters::driving::messaging::MessagedEvent;


#[derive(Clone)]
pub struct OutboxRepo {}

impl OutboxRepo {
    pub async fn save(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &MessagedEvent,
    ) -> Result<(), Error> {
        let id = event.id().to_string();
        let view = event.view();
        let name = event.name();
        sqlx::query("INSERT INTO outbox (id, name) VALUES ($1, $2)")
            .bind(&id())
            .bind(name)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}