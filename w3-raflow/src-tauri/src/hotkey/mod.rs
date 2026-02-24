// src-tauri/src/hotkey/mod.rs

//! 全局热键模块

pub mod manager;
pub mod handler;

pub use manager::HotkeyManager;
pub use handler::HotkeyHandler;
