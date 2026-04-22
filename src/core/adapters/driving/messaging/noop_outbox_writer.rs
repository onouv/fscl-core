use std::future::{Future, ready};

use fscl_messaging::EventEnvelope;

use super::OutboxWriter;

// Test helper writer that accepts envelopes but does not persist them.
#[derive(Clone, Default)]
pub struct NoopOutboxWriter;

impl OutboxWriter for NoopOutboxWriter {
    type Error = String;
    type Tx<'tx> = ();

    fn append(
        &self,
        _tx: &mut Self::Tx<'_>,
        _envelope: EventEnvelope,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
}
