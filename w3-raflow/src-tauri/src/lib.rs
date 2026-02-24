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

// Re-export commonly used types
pub use core::app::RaFlowApp;
pub use config::{ConfigStorage, AppConfig};
pub use audio::AudioService;
pub use hotkey::{HotkeyManager, HotkeyHandler};
