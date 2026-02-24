// src-tauri/src/hotkey/manager.rs

//! 热键管理器

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::Result;
use crate::config::HotkeyConfig;

/// 热键管理器
pub struct HotkeyManager {
    /// 当前热键配置
    config: Arc<Mutex<HotkeyConfig>>,

    /// 是否已注册
    registered: Arc<Mutex<bool>>,
}

impl HotkeyManager {
    /// 创建新的热键管理器
    pub fn new(config: HotkeyConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            registered: Arc::new(Mutex::new(false)),
        }
    }

    /// 注册热键
    pub async fn register(&self) -> Result<()> {
        let config = self.config.lock().await;

        if !config.enabled {
            tracing::info!("Hotkey is disabled, skipping registration");
            return Ok(());
        }

        tracing::info!("Registering hotkey: {:?}", config);

        // 构建热键字符串
        let hotkey_str = self.build_hotkey_string(&config)?;

        // TODO: 使用 Tauri 的全局快捷键插件注册
        // 这需要在 Tauri 的上下文中完成
        // tauri_plugin_global_shortcut::Builder::new()
        //     .with_handler(|_app, _shortcut, _event| {
        //         // 触发录音
        //     })
        //     .build();

        tracing::info!("Hotkey registered: {}", hotkey_str);
        *self.registered.lock().await = true;

        Ok(())
    }

    /// 注销热键
    pub async fn unregister(&self) -> Result<()> {
        tracing::info!("Unregistering hotkey");

        *self.registered.lock().await = false;

        Ok(())
    }

    /// 更新热键配置
    pub async fn update_config(&self, config: HotkeyConfig) -> Result<()> {
        // 先注销旧的
        if *self.registered.lock().await {
            self.unregister().await?;
        }

        // 重新注册
        if config.enabled {
            self.register().await?;
        }

        // 更新配置
        *self.config.lock().await = config;

        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> HotkeyConfig {
        self.config.lock().await.clone()
    }

    /// 是否已注册
    pub async fn is_registered(&self) -> bool {
        *self.registered.lock().await
    }

    /// 构建热键字符串
    fn build_hotkey_string(&self, config: &HotkeyConfig) -> Result<String> {
        let mut parts = Vec::new();

        // 添加修饰键
        for modifier in &config.modifiers {
            let mod_str = match modifier {
                crate::config::KeyModifier::Ctrl => "Ctrl",
                crate::config::KeyModifier::Alt => "Alt",
                crate::config::KeyModifier::Shift => "Shift",
                crate::config::KeyModifier::Super => "Super",
            };
            parts.push(mod_str.to_string());
        }

        // 添加主键
        let key_str = match &config.key {
            crate::config::KeyCode::Backslash => "\\",
            crate::config::KeyCode::Space => "Space",
            crate::config::KeyCode::Char(c) => &c.to_string(),
        };
        parts.push(key_str.to_string());

        Ok(parts.join("+"))
    }
}

/// 热键触发器（由 Tauri 命令调用）
pub struct HotkeyTrigger {
    /// 触发回调
    on_trigger: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl HotkeyTrigger {
    /// 创建新的触发器
    pub fn new() -> Self {
        Self {
            on_trigger: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置回调
    pub fn set_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        *self.on_trigger.blocking_lock() = Some(Box::new(callback));
    }

    /// 触发热键
    pub fn trigger(&self) {
        if let Some(callback) = self.on_trigger.blocking_lock().as_ref() {
            callback();
        }
    }
}

impl Default for HotkeyTrigger {
    fn default() -> Self {
        Self::new()
    }
}
