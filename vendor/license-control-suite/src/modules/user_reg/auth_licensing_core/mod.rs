//! Core auth and licensing business rules.
//!
//! This module intentionally has no Tauri dependency. Desktop IPC, keychain,
//! filesystem, and HTTP adapters live outside this module.

mod domain;
mod service;
mod state;
mod traits;

pub use domain::*;
pub use service::*;
pub use state::*;
pub use traits::*;

pub mod test_support;
