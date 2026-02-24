// src-tauri/src/input/mod.rs

//! 输入模块

pub mod keyboard;
pub mod clipboard;
pub mod service;

pub use service::{TextService, TextInsertionResult};
pub use keyboard::KeyboardSimulator;
pub use clipboard::ClipboardService;
