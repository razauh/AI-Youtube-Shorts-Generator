//! Tauri integration for auth and licensing.
//!
//! This module owns IPC DTOs and production adapters. Business rules live in
//! `auth_licensing_core`.

#[cfg(feature = "desktop-tauri")]
mod commands;
#[cfg(feature = "desktop-tauri")]
mod http_client;
#[cfg(feature = "desktop-persistence")]
mod persistence;

#[cfg(feature = "desktop-tauri")]
pub use commands::*;
#[cfg(feature = "desktop-tauri")]
pub use http_client::*;
#[cfg(feature = "desktop-persistence")]
pub use persistence::*;
