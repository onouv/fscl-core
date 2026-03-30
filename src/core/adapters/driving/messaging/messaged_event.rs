use derive_getters::Getters;
use serde::{Serialize};
use serde_json::*;

//use super::ComponentDTO;

#[derive(Serialize, Getters)]
pub(super) struct MessagedEvent {
    event_type: String,
    aggregate_type: String,
    aggregate_id: String,
    view_id: String,
    payload: Option<serde_json::Value>,
}

impl MessagedEvent {
    fn default() -> Self {
        Self {
            event_type: String::new(),
            aggregate_type: String::new(),
            aggregate_id: String::new(),
            view_id: String::new(),
            payload: None 
        }
    }

    fn with_event_type(mut self, event_type: &str) -> Self {
        self.event_type = event_type.to_string();
        self
    }
    
    fn with_aggregate_type(mut self, aggregate_type: &str) -> Self {
        self.aggregate_type = aggregate_type.to_string();
        self
    }
    
    fn with_aggregate_id(mut self, id: &str) -> Self {
        self.aggregate_id = id.to_string();
        self
    }

    fn with_view_id(mut self, id: &str) -> Self {
        self.view_id = id.to_string();
        self
    }

    fn with_payload<T: Serialize>(mut self, payload: T) -> Result<Self> {
        self.payload = match serde_json::to_value::<T>(payload) {
            Ok(v) => Some(v),
            Err(e) => {
                return Err(e);
            }
        };

        Ok(self)
    }
}



