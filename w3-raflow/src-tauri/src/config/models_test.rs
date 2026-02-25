// src-tauri/src/config/models_test.rs

//! 配置模型单元测试

#[cfg(test)]
mod tests {
    use super::super::models::*;
    use serde_json;

    // GeneralConfig 测试
    #[test]
    fn test_general_config_default() {
        let config = GeneralConfig::default();
        assert_eq!(config.language, "zh-CN");
        assert_eq!(config.autostart, false);
        assert_eq!(config.minimize_to_tray, true);
    }

    #[test]
    fn test_general_config_serialize() {
        let config = GeneralConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("zh-CN"));
        assert!(json.contains("autostart"));
        assert!(json.contains("minimize_to_tray"));
    }

    #[test]
    fn test_general_config_deserialize() {
        let json = r#"{
            "language": "en-US",
            "autostart": true,
            "minimize_to_tray": false
        }"#;

        let config: GeneralConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.language, "en-US");
        assert_eq!(config.autostart, true);
        assert_eq!(config.minimize_to_tray, false);
    }

    // AudioConfig 测试
    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.device_id, "");
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.echo_cancellation, true);
        assert_eq!(config.noise_suppression, true);
        assert_eq!(config.auto_gain, true);
    }

    #[test]
    fn test_audio_config_serialize() {
        let config = AudioConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("16000"));
        assert!(json.contains("echo_cancellation"));
    }

    #[test]
    fn test_audio_config_with_custom_device() {
        let config = AudioConfig {
            device_id: "device-123".to_string(),
            ..Default::default()
        };
        assert_eq!(config.device_id, "device-123");
    }

    // ElevenLabsConfig 测试
    #[test]
    fn test_elevenlabs_config_default() {
        let config = ElevenLabsConfig::default();
        assert_eq!(config.api_key, "");
        assert_eq!(config.language, "auto");
        assert_eq!(config.timeout, 30);
    }

    #[test]
    fn test_elevenlabs_config_serialize() {
        let config = ElevenLabsConfig {
            api_key: "xi-test-key".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("xi-test-key"));
    }

    // HotkeyConfig 测试
    #[test]
    fn test_hotkey_config_default() {
        let config = HotkeyConfig::default();
        assert_eq!(config.modifiers.len(), 2);
        assert!(config.modifiers.contains(&KeyModifier::Ctrl));
        assert!(config.modifiers.contains(&KeyModifier::Shift));
        assert_eq!(config.key, KeyCode::Char('o'));
        assert_eq!(config.enabled, true);
    }

    #[test]
    fn test_key_modifier_serialize() {
        let modifier = KeyModifier::Super;
        let json = serde_json::to_string(&modifier).unwrap();
        assert_eq!(json, "\"super\"");
    }

    #[test]
    fn test_key_modifier_deserialize() {
        let json = "\"ctrl\"";
        let modifier: KeyModifier = serde_json::from_str(json).unwrap();
        assert_eq!(modifier, KeyModifier::Ctrl);
    }

    #[test]
    fn test_all_key_modifiers() {
        let modifiers = vec![
            KeyModifier::Ctrl,
            KeyModifier::Alt,
            KeyModifier::Shift,
            KeyModifier::Super,
        ];

        for modifier in modifiers {
            let json = serde_json::to_string(&modifier).unwrap();
            let deserialized: KeyModifier = serde_json::from_str(&json).unwrap();
            assert_eq!(modifier, deserialized);
        }
    }

    #[test]
    fn test_key_code_serialize() {
        let code = KeyCode::Char('a');
        let json = serde_json::to_string(&code).unwrap();
        // Char variants serialize to {"char":"a"}
        assert_eq!(json, "{\"char\":\"a\"}");
    }

    #[test]
    fn test_key_code_backslash_serialize() {
        let code = KeyCode::Backslash;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"backslash\"");
    }

    #[test]
    fn test_key_code_space_serialize() {
        let code = KeyCode::Space;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"space\"");
    }

    #[test]
    fn test_key_code_deserialize() {
        let json = "{\"char\":\"a\"}";
        let code: KeyCode = serde_json::from_str(json).unwrap();
        assert_eq!(code, KeyCode::Char('a'));
    }

    // TextConfig 测试
    #[test]
    fn test_text_config_default() {
        let config = TextConfig::default();
        assert_eq!(config.strategy, "auto");
        assert_eq!(config.insertion_delay, 100);
    }

    #[test]
    fn test_text_config_serialize() {
        let config = TextConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("auto"));
        assert!(json.contains("100"));
    }

    // UIConfig 测试
    #[test]
    fn test_ui_config_default() {
        let config = UIConfig::default();
        assert_eq!(config.show_notifications, true);
        assert_eq!(config.indicator_opacity, 0.9);
        assert_eq!(config.show_live_preview, true);
    }

    #[test]
    fn test_ui_config_opacity_bounds() {
        let config = UIConfig {
            indicator_opacity: 0.0,
            ..Default::default()
        };
        assert_eq!(config.indicator_opacity, 0.0);

        let config = UIConfig {
            indicator_opacity: 1.0,
            ..Default::default()
        };
        assert_eq!(config.indicator_opacity, 1.0);
    }

    // AppConfig 测试
    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.general.language, "zh-CN");
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.elevenlabs.language, "auto");
        assert_eq!(config.hotkey.enabled, true);
        assert_eq!(config.text.strategy, "auto");
        assert_eq!(config.ui.show_notifications, true);
    }

    #[test]
    fn test_app_config_serialize() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("general"));
        assert!(json.contains("audio"));
        assert!(json.contains("elevenlabs"));
        assert!(json.contains("hotkey"));
        assert!(json.contains("text"));
        assert!(json.contains("ui"));
    }

    #[test]
    fn test_app_config_deserialize() {
        let json = r#"{
            "general": {
                "language": "en-US",
                "autostart": false,
                "minimize_to_tray": true
            },
            "audio": {
                "device_id": "",
                "sample_rate": 16000,
                "echo_cancellation": true,
                "noise_suppression": true,
                "auto_gain": true
            },
            "elevenlabs": {
                "api_key": "",
                "language": "auto",
                "timeout": 30
            },
            "hotkey": {
                "modifiers": ["ctrl", "shift"],
                "key": {"char": "o"},
                "enabled": true
            },
            "text": {
                "strategy": "auto",
                "insertion_delay": 100
            },
            "ui": {
                "show_notifications": true,
                "indicator_opacity": 0.9,
                "show_live_preview": true
            }
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.general.language, "en-US");
        assert_eq!(config.hotkey.key, KeyCode::Char('o'));
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.general.language, config.general.language);
        assert_eq!(cloned.audio.sample_rate, config.audio.sample_rate);
    }

    // 修饰键组合测试
    #[test]
    fn test_modifier_combinations() {
        let combinations = vec![
            vec![KeyModifier::Ctrl],
            vec![KeyModifier::Ctrl, KeyModifier::Shift],
            vec![KeyModifier::Ctrl, KeyModifier::Alt, KeyModifier::Shift],
            vec![KeyModifier::Super],
        ];

        for modifiers in combinations {
            let json = serde_json::to_string(&modifiers).unwrap();
            let deserialized: Vec<KeyModifier> = serde_json::from_str(&json).unwrap();
            assert_eq!(modifiers, deserialized);
        }
    }

    // 特殊按键测试
    #[test]
    fn test_special_key_codes() {
        let codes = vec![
            KeyCode::Backslash,
            KeyCode::Space,
            KeyCode::Char('z'),
            KeyCode::Char('0'),
            KeyCode::Char('9'),
        ];

        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            let deserialized: KeyCode = serde_json::from_str(&json).unwrap();
            assert_eq!(code, deserialized);
        }
    }

    // TOML 序列化测试
    #[test]
    fn test_config_toml_serialize() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("[audio]"));
        assert!(toml_str.contains("[elevenlabs]"));
        assert!(toml_str.contains("[hotkey]"));
        assert!(toml_str.contains("[text]"));
        assert!(toml_str.contains("[ui]"));
    }

    #[test]
    fn test_config_toml_deserialize() {
        let toml_str = r#"
            [general]
            language = "zh-CN"
            autostart = false
            minimize_to_tray = true

            [audio]
            device_id = ""
            sample_rate = 16000
            echo_cancellation = true
            noise_suppression = true
            auto_gain = true

            [elevenlabs]
            api_key = ""
            language = "auto"
            timeout = 30

            [hotkey]
            modifiers = ["ctrl", "shift"]
            key = { char = "o" }
            enabled = true

            [text]
            strategy = "auto"
            insertion_delay = 100

            [ui]
            show_notifications = true
            indicator_opacity = 0.9
            show_live_preview = true
        "#;

        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.general.language, "zh-CN");
        assert_eq!(config.audio.sample_rate, 16000);
    }

    // 边界值测试
    #[test]
    fn test_sample_rate_bounds() {
        let config = AudioConfig {
            sample_rate: 8000,
            ..Default::default()
        };
        assert_eq!(config.sample_rate, 8000);

        let config = AudioConfig {
            sample_rate: 48000,
            ..Default::default()
        };
        assert_eq!(config.sample_rate, 48000);
    }

    #[test]
    fn test_timeout_bounds() {
        let config = ElevenLabsConfig {
            timeout: 1,
            ..Default::default()
        };
        assert_eq!(config.timeout, 1);

        let config = ElevenLabsConfig {
            timeout: 300,
            ..Default::default()
        };
        assert_eq!(config.timeout, 300);
    }

    #[test]
    fn test_insertion_delay_bounds() {
        let config = TextConfig {
            insertion_delay: 0,
            ..Default::default()
        };
        assert_eq!(config.insertion_delay, 0);

        let config = TextConfig {
            insertion_delay: 1000,
            ..Default::default()
        };
        assert_eq!(config.insertion_delay, 1000);
    }
}
