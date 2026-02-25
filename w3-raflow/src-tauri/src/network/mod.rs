// src-tauri/src/network/mod.rs

//! 网络模块

pub mod protocol;
pub mod websocket;
pub mod transcription;
pub mod optimize;

// Include test modules
#[cfg(test)]
mod protocol_test;

pub use protocol::{ClientMessage, ServerMessage, TranscriptionResult};
pub use transcription::TranscriptionService;
pub use optimize::{Base64Encoder, Base64Config, MessageBatcher, ConnectionManager};
