//! Legacy compatibility auth/licensing domain.
//!
//! This module remains public for transitional compatibility. New client
//! licensing integrations should prefer `license_control_suite::core`.

pub const NAMESPACE: &str = "auth_core";

pub mod adapters;
pub mod auth;
pub mod compatibility;
pub mod models;
pub mod policy;
pub mod reset;
pub mod session;
