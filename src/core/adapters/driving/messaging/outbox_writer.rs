use std::future::Future;

use fscl_messaging::EventEnvelope;

pub trait OutboxWriter: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn append(
        &self,
        tx: &mut Self::Tx<'_>,
        envelope: EventEnvelope,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
