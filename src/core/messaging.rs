use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{IdFormat, ResourceId};

use fscl_messaging::{AggregateType, EventEnvelope, EventEnvelopeError};

pub use fscl_messaging::{
    OUTBOX_NOTIFY_CHANNEL, OUTBOX_SCHEMA_SQL, OUTBOX_SCHEMA_VERSION, OUTBOX_TABLE,
};

pub fn build_event_envelope<T: Serialize>(
    occurred_at: DateTime<Utc>,
    event_type: impl Into<String>,
    aggregate_type: AggregateType,
    aggregate_id: &ResourceId,
    view_id: impl Into<String>,
    payload: T,
) -> Result<EventEnvelope, EventEnvelopeError> {
    EventEnvelope::builder()
        .with_occurred_at(occurred_at)
        .with_event_type(event_type)
        .with_aggregate_type(aggregate_type)
        .with_aggregate_id(aggregate_id.as_str())
        .with_view_id(view_id)
        .build(payload)
}

#[cfg(test)]
mod tests {
    use chrono::{Utc, format};
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct Payload {
        name: &'static str,
    }

    #[test]
    fn builds_envelope_from_resource_id() {
        let format = IdFormat::new(None, None, None).unwrap();
        let resource_id =
            ResourceId::new("component-1".to_string(), format).expect("resource id should build");

        let envelope = build_event_envelope(
            Utc::now(),
            "created",
            AggregateType::Component,
            &resource_id,
            "process",
            Payload { name: "Compressor" },
        )
        .expect("envelope should build");

        assert_eq!(envelope.aggregate_id, "component-1");
        assert_eq!(envelope.subject("events"), "events.component.created");
    }
}
