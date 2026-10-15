//! Deterministic NSE Cash Market data plane.
//!
//! Raw exchange bytes enter the system here. Versioned canonical artifacts are
//! the only supported boundary to downstream Python research code.

#![forbid(unsafe_code)]

pub mod book;
pub mod events;
pub mod jiffy;
pub mod layouts;
pub mod lifecycle;
pub mod manifest;
pub mod parser;
pub mod quarantine;
pub mod reconcile;
