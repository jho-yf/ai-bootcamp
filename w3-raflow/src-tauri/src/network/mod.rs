// src-tauri/src/network/mod.rs

//! 网络模块

pub mod protocol;
pub mod websocket;
pub mod transcription;

pub use protocol::{ClientMessage, ServerMessage, TranscriptionResult};
pub use transcription::TranscriptionService;
