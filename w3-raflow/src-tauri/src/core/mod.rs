// src-tauri/src/core/mod.rs

pub mod error;
pub mod state;
pub mod app;

// Include test modules
#[cfg(test)]
mod error_test;
#[cfg(test)]
mod state_test;

pub use error::{AppError, Result};
pub use state::{AppState, RecordingState, ConnectionState};
