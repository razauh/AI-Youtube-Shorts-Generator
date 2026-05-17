//! Curated desktop persistence facade.

pub use crate::modules::user_reg::auth_licensing_tauri::{
    AppDataStateStore, InMemorySecretStore, KeychainSecretStore,
};
