// src-tauri/src/lib.rs

//! RaFlow Library
//!
//! RaFlow is a real-time voice input tool powered by ElevenLabs Scribe v2.

// Re-export public modules
pub mod core;
pub mod audio;
pub mod network;
pub mod input;
pub mod config;
pub mod tray;
pub mod commands;
pub mod hotkey;
pub mod updater;

// Re-export commonly used types
pub use core::app::RaFlowApp;
pub use core::{AppState, RecordingState, ConnectionState};
pub use core::{AppError, Result, AudioError, NetworkError, ConfigError, InputError};
pub use config::{ConfigStorage, AppConfig};
pub use audio::AudioService;
pub use network::{ClientMessage, ServerMessage, TranscriptionResult};
pub use hotkey::{HotkeyManager, HotkeyHandler};
pub use updater::{UpdateManager, UpdateStatus, UpdateInfo};
