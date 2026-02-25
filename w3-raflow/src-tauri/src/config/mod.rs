// src-tauri/src/config/mod.rs

pub mod models;
pub mod storage;

// Include test modules
#[cfg(test)]
mod models_test;
#[cfg(test)]
mod storage_test;

pub use models::*;
pub use storage::ConfigStorage;
