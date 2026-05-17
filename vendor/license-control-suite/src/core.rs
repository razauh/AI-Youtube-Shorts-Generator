//! Curated crate-level core API for desktop integrators.
//!
//! This is the canonical client auth/licensing path for new consumers. It
//! points at the trait-oriented `user_reg::auth_licensing_core` surface without
//! requiring deep `modules::...` imports.

pub use crate::modules::user_reg::auth_licensing_core::*;
