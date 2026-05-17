//! Curated desktop-only admin console facade.
//!
//! This surface exposes the existing admin domain as a desktop-only admin
//! console boundary. It is intentionally separate from the six end-user/client
//! licensing commands and is not a web dashboard target.

pub use crate::modules::admin_dashboard::{
    adapters, auth, authz, compatibility, ops, queue, realtime,
};
