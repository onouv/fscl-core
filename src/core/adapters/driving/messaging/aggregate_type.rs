use serde::Serialize;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, EnumString, Serialize)]
pub enum AggregateType {
    FUNCTION,
    COMPONENT,
    SYSYTEM,
    LOCATION
}

