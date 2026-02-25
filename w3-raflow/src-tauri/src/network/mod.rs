// src-tauri/src/network/mod.rs

//! 网络模块

pub mod protocol;
pub mod websocket;
pub mod transcription;

// Include test modules
#[cfg(test)]
mod protocol_test;

pub use protocol::{ClientMessage, ServerMessage, TranscriptionResult};
pub use transcription::TranscriptionService;
