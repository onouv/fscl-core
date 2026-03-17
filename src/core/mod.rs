mod domain;
pub use domain::*;

pub mod application;
pub use application::*;

pub mod ports;
pub use ports::*;

pub mod adapters;

// just for demonstrating use of the core lib
// TODO: move this to a cargo doc scraped example
mod demo_client;