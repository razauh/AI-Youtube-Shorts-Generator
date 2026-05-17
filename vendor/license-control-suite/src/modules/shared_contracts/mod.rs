//! Shared contract models for compatibility and admin flows.
//!
//! This module remains public for compatibility and supporting domains, but it
//! is not the default new-consumer client auth surface.

pub const NAMESPACE: &str = "shared_contracts";

pub mod dto;
pub mod errors;
pub mod events;
pub mod state;
pub mod versioning;
