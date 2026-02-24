// src-tauri/src/core/mod.rs

pub mod error;
pub mod state;
pub mod app;

pub use error::{AppError, Result};
pub use state::{AppState, RecordingState, ConnectionState};
