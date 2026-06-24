//! Curated reference-backend facade.
//!
//! This surface exposes the local/native reference backend used for local
//! testing and route-contract reasoning. It is not a claim of Cloudflare
//! runtime readiness, Gumroad provider support, payment verification, webhook
//! ingestion, or durable backend storage.

pub use crate::modules::user_reg::licensing_worker::*;
