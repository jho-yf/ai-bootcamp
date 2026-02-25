// src-tauri/src/config/storage.rs

//! 配置存储服务

use std::path::PathBuf;
use std::io::Write;

use crate::core::{AppError, Result, error::ConfigError};
use super::AppConfig;

/// 配置存储服务
pub struct ConfigStorage {
    /// 配置文件路径
    config_path: PathBuf,
}

impl ConfigStorage {
    /// 创建新的配置存储实例
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
                AppError::Config(ConfigError::DirectoryInaccessible(
                    "无法获取配置目录".to_string()
                ))
            })?;

        let config_path = config_dir.join("raflow").join("config.toml");

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Config(ConfigError::DirectoryInaccessible(
                    format!("创建配置目录失败: {}", e)
                ))
            })?;
        }

        Ok(Self { config_path })
    }

    /// 使用指定路径创建配置存储实例（主要用于测试）
    #[cfg(test)]
    pub fn new_with_path(config_path: PathBuf) -> Result<Self> {
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Config(ConfigError::DirectoryInaccessible(
                    format!("创建配置目录失败: {}", e)
                ))
            })?;
        }

        Ok(Self { config_path })
    }

    /// 加载配置
    pub fn load(&self) -> Result<AppConfig> {
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path).map_err(|e| {
                AppError::Config(ConfigError::LoadFailed(
                    format!("读取配置文件失败: {}", e)
                ))
            })?;

            let config: AppConfig = toml::from_str(&content).map_err(|e| {
                AppError::Config(ConfigError::Corrupted(
                    format!("解析配置文件失败: {}", e)
                ))
            })?;

            Ok(config)
        } else {
            // 配置文件不存在，返回默认配置
            Ok(AppConfig::default())
        }
    }

    /// 保存配置
    pub fn save(&self, config: &AppConfig) -> Result<()> {
        let content = toml::to_string_pretty(config).map_err(|e| {
            AppError::Config(ConfigError::SaveFailed(
                format!("序列化配置失败: {}", e)
            ))
        })?;

        // 先写入临时文件
        let temp_path = self.config_path.with_extension("tmp");
        let mut file = std::fs::File::create(&temp_path).map_err(|e| {
            AppError::Config(ConfigError::SaveFailed(
                format!("创建临时配置文件失败: {}", e)
            ))
        })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            AppError::Config(ConfigError::SaveFailed(
                format!("写入配置文件失败: {}", e)
            ))
        })?;

        // 原子性替换
        std::fs::rename(&temp_path, &self.config_path).map_err(|e| {
            AppError::Config(ConfigError::SaveFailed(
                format!("保存配置文件失败: {}", e)
            ))
        })?;

        Ok(())
    }

    /// 获取配置文件路径
    pub fn path(&self) -> &PathBuf {
        &self.config_path
    }

    /// 重置为默认配置
    pub fn reset(&self) -> Result<AppConfig> {
        let default = AppConfig::default();
        self.save(&default)?;
        Ok(default)
    }
}

impl Default for ConfigStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigStorage")
    }
}
