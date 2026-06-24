//! Curated crate-level desktop API.
//!
//! Tauri command wiring and persistence adapters live under this namespace.

#[cfg(feature = "core")]
pub mod admin;
#[cfg(feature = "desktop-persistence")]
pub mod persistence;
#[cfg(feature = "desktop-tauri")]
pub mod tauri;
