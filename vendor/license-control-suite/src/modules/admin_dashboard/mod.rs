//! Desktop admin domain.
//!
//! This module remains a separate desktop admin surface and is not the
//! canonical client auth core.

pub const NAMESPACE: &str = "admin_dashboard";

pub mod adapters;
pub mod auth;
pub mod authz;
pub mod compatibility;
pub mod ops;
pub mod queue;
pub mod realtime;
