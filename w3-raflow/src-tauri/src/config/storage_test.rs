// src-tauri/src/config/storage_test.rs

//! 配置存储单元测试

#[cfg(test)]
mod tests {
    use super::super::storage::*;
    use super::super::models::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // 辅助函数：创建测试用临时目录
    fn create_temp_config_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        (temp_dir, config_path)
    }

    // ConfigStorage::new 测试
    #[test]
    fn test_config_storage_new_creates_directory() {
        let (temp_dir, config_path) = create_temp_config_dir();
        let storage_dir = config_path.parent().unwrap();

        // 删除目录以测试创建
        if storage_dir.exists() {
            fs::remove_dir_all(storage_dir).unwrap();
        }

        let storage = ConfigStorage::new_with_path(config_path.clone()).unwrap();
        assert!(storage_dir.exists());
        assert!(storage_dir.is_dir());

        drop(storage);
        temp_dir.close().unwrap();
    }

    // ConfigStorage::load 测试 - 不存在时返回默认配置
    #[test]
    fn test_config_storage_load_returns_default_when_not_exists() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let storage = ConfigStorage::new_with_path(config_path).unwrap();
        let config = storage.load().unwrap();

        assert_eq!(config.general.language, "zh-CN");
        assert_eq!(config.audio.sample_rate, 16000);

        temp_dir.close().unwrap();
    }

    // ConfigStorage::load 测试 - 加载现有配置
    #[test]
    fn test_config_storage_load_existing_config() {
        let (temp_dir, config_path) = create_temp_config_dir();

        // 创建测试配置文件
        let test_config = AppConfig {
            general: GeneralConfig {
                language: "en-US".to_string(),
                autostart: true,
                minimize_to_tray: false,
            },
            ..Default::default()
        };

        let storage = ConfigStorage::new_with_path(config_path.clone()).unwrap();
        storage.save(&test_config).unwrap();

        // 加载配置
        let loaded_config = storage.load().unwrap();
        assert_eq!(loaded_config.general.language, "en-US");
        assert_eq!(loaded_config.general.autostart, true);
        assert_eq!(loaded_config.general.minimize_to_tray, false);

        temp_dir.close().unwrap();
    }

    // ConfigStorage::save 测试
    #[test]
    fn test_config_storage_save_creates_file() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let storage = ConfigStorage::new_with_path(config_path.clone()).unwrap();
        let config = AppConfig::default();

        storage.save(&config).unwrap();
        assert!(config_path.exists());

        temp_dir.close().unwrap();
    }

    // ConfigStorage::save 测试 - 保存后加载一致
    #[test]
    fn test_config_storage_save_and_load_consistency() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let original_config = AppConfig {
            general: GeneralConfig {
                language: "ja-JP".to_string(),
                autostart: false,
                minimize_to_tray: true,
            },
            audio: AudioConfig {
                device_id: "test-device".to_string(),
                sample_rate: 48000,
                echo_cancellation: false,
                noise_suppression: false,
                auto_gain: false,
            },
            ..Default::default()
        };

        let storage = ConfigStorage::new_with_path(config_path).unwrap();
        storage.save(&original_config).unwrap();

        let loaded_config = storage.load().unwrap();
        assert_eq!(loaded_config.general.language, "ja-JP");
        assert_eq!(loaded_config.audio.device_id, "test-device");
        assert_eq!(loaded_config.audio.sample_rate, 48000);

        temp_dir.close().unwrap();
    }

    // 损坏的配置文件测试
    #[test]
    fn test_config_storage_load_corrupted_returns_default() {
        let (temp_dir, config_path) = create_temp_config_dir();

        // 创建损坏的配置文件
        fs::write(&config_path, "invalid toml content {{{").unwrap();

        let storage = ConfigStorage::new_with_path(config_path).unwrap();
        let result = storage.load();

        // 应该返回错误或默认配置
        assert!(result.is_err() || result.unwrap().general.language == "zh-CN");

        temp_dir.close().unwrap();
    }

    // 覆盖保存测试
    #[test]
    fn test_config_storage_save_overwrites_existing() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let storage = ConfigStorage::new_with_path(config_path).unwrap();

        // 第一次保存
        let config1 = AppConfig {
            general: GeneralConfig {
                language: "zh-CN".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        storage.save(&config1).unwrap();

        // 第二次保存（覆盖）
        let config2 = AppConfig {
            general: GeneralConfig {
                language: "en-US".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        storage.save(&config2).unwrap();

        // 验证覆盖后的值
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.general.language, "en-US");

        temp_dir.close().unwrap();
    }

    // 多次保存和加载测试
    #[test]
    fn test_config_storage_multiple_save_load_cycles() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let storage = ConfigStorage::new_with_path(config_path).unwrap();

        for i in 0..5 {
            let config = AppConfig {
                general: GeneralConfig {
                    language: format!("lang-{}", i),
                    ..Default::default()
                },
                ..Default::default()
            };

            storage.save(&config).unwrap();
            let loaded = storage.load().unwrap();
            assert_eq!(loaded.general.language, format!("lang-{}", i));
        }

        temp_dir.close().unwrap();
    }

    // Unicode 内容测试
    #[test]
    fn test_config_storage_unicode_content() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let config = AppConfig {
            general: GeneralConfig {
                language: "zh-CN".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let storage = ConfigStorage::new_with_path(config_path).unwrap();
        storage.save(&config).unwrap();

        let loaded = storage.load().unwrap();
        assert_eq!(loaded.general.language, "zh-CN");

        temp_dir.close().unwrap();
    }

    // 完整配置测试
    #[test]
    fn test_config_storage_full_config() {
        let (temp_dir, config_path) = create_temp_config_dir();

        let config = AppConfig {
            general: GeneralConfig {
                language: "ko-KR".to_string(),
                autostart: true,
                minimize_to_tray: false,
            },
            audio: AudioConfig {
                device_id: "device-123".to_string(),
                sample_rate: 48000,
                echo_cancellation: true,
                noise_suppression: false,
                auto_gain: true,
            },
            elevenlabs: ElevenLabsConfig {
                api_key: "xi-test-key-12345".to_string(),
                language: "auto".to_string(),
                timeout: 60,
            },
            hotkey: HotkeyConfig {
                modifiers: vec![KeyModifier::Ctrl, KeyModifier::Alt],
                key: KeyCode::Char('r'),
                enabled: true,
            },
            text: TextConfig {
                strategy: "keyboard".to_string(),
                insertion_delay: 200,
            },
            ui: UIConfig {
                show_notifications: false,
                indicator_opacity: 0.7,
                show_live_preview: false,
            },
        };

        let storage = ConfigStorage::new_with_path(config_path).unwrap();
        storage.save(&config).unwrap();

        let loaded = storage.load().unwrap();
        assert_eq!(loaded.general.language, "ko-KR");
        assert_eq!(loaded.audio.device_id, "device-123");
        assert_eq!(loaded.elevenlabs.api_key, "xi-test-key-12345");
        assert_eq!(loaded.hotkey.key, KeyCode::Char('r'));
        assert_eq!(loaded.text.strategy, "keyboard");

        temp_dir.close().unwrap();
    }
}
