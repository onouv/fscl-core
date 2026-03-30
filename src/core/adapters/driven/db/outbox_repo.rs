use sqlx::{Error, Transaction, postgres::Postgres};
use crate::adapters::Message;


#[derive(Clone)]
pub struct OutboxRepo {}

impl OutboxRepo {
    pub async fn save(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        message: &Message,
    ) -> Result<(), Error> {
        let id = message.id().to_string();
        let view = message.view();
        let name = message.name();
        sqlx::query("INSERT INTO outbox (id, name) VALUES ($1, $2)")
            .bind(&id())
            .bind(name)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}