pub const NAMESPACE: &str = "user_reg";

#[cfg(feature = "core")]
pub mod auth_licensing_core;
#[cfg(any(feature = "desktop-tauri", feature = "desktop-persistence"))]
pub mod auth_licensing_tauri;
#[cfg(feature = "reference-worker")]
pub mod licensing_worker;
