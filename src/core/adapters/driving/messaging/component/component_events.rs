use super::MessagedEvent;
pub struct ComponentCreatedEvent {
    inner: MessagedEvent,
}

impl ComponentCreatedEvent {
    pub fn new(component: ComponentDTO, view_id: &str) -> Result<Self> {
        Ok(Self {
            inner: MessagedEvent::default()
                .with_event_type("created")
                .with_aggregate_type("component")
                .with_aggregate_id(component.id())
                .with_view_id(view_id)
                .with_payload::<ComponentDTO>(component)?,
        })
    }

    pub fn to_event(self) -> MessagedEvent {
        self.inner
    } 
}

/* 
pub struct ComponentDeletedEvent {}

impl ComponentDeletedEvent {
    pub fn to_event(component_id: String, view_id: &str) -> Event {
        Event::default()
            .with_event_type("deleted")
            .with_aggregate_type("component")
            .with_aggregate_id(&component_id)
            .with_view_id(view_id)
    }
}
*/