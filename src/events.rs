use std::borrow::Cow;

/// Enumeration of the various different event types emitted by the parser
pub enum EventType {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
}

pub struct Event<'a> {
    /// The type of the event
    pub event_type: EventType,

    /// An in-document path for relating to the event
    pub path: Cow<'a, str>,
}
