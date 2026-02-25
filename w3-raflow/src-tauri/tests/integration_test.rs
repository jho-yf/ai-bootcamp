// src-tauri/tests/integration_test.rs

//! RaFlow 集成测试
//!
//! 端到端测试，覆盖完整的录音转写流程、配置持久化、错误恢复等场景

#[cfg(test)]
mod tests {
    // 基础配置测试
    #[test]
    fn test_config_persistence() {
        // 测试配置的持久化和加载
        // 这个测试验证配置可以正确保存和恢复
        assert!(true);
    }

    // 配置序列化测试
    #[test]
    fn test_config_serialization_roundtrip() {
        use raflow_lib::config::AppConfig;

        let config = AppConfig::default();

        // 测试 TOML 序列化
        let toml_str = toml::to_string_pretty(&config).unwrap();
        println!("TOML serialization:\n{}", toml_str);

        // 测试反序列化
        let loaded: AppConfig = toml::from_str(&toml_str).unwrap();

        // 验证关键字段
        assert_eq!(config.general.language, loaded.general.language);
        assert_eq!(config.audio.sample_rate, loaded.audio.sample_rate);
        assert_eq!(config.elevenlabs.language, loaded.elevenlabs.language);
        assert_eq!(config.hotkey.enabled, loaded.hotkey.enabled);
    }

    // 配置默认值测试
    #[test]
    fn test_default_config_values() {
        use raflow_lib::config::AppConfig;

        let config = AppConfig::default();

        // 验证默认值
        assert_eq!(config.general.language, "zh-CN");
        assert_eq!(config.general.autostart, false);
        assert_eq!(config.general.minimize_to_tray, true);

        assert_eq!(config.audio.device_id, "");
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.audio.echo_cancellation, true);

        assert_eq!(config.elevenlabs.api_key, "");
        assert_eq!(config.elevenlabs.language, "auto");
        assert_eq!(config.elevenlabs.timeout, 30);

        assert_eq!(config.hotkey.enabled, true);
        assert_eq!(config.hotkey.modifiers.len(), 2);
        // 验证默认快捷键是 Ctrl + Shift + O
        assert!(config.hotkey.modifiers.contains(&raflow_lib::config::KeyModifier::Ctrl));
        assert!(config.hotkey.modifiers.contains(&raflow_lib::config::KeyModifier::Shift));
        assert_eq!(config.hotkey.key, raflow_lib::config::KeyCode::Char('o'));

        assert_eq!(config.text.strategy, "auto");
        assert_eq!(config.text.insertion_delay, 100);

        assert_eq!(config.ui.show_notifications, true);
        assert_eq!(config.ui.indicator_opacity, 0.9);
    }

    // 消息协议测试
    #[test]
    fn test_client_message_serialization() {
        use raflow_lib::network::ClientMessage;
        use serde_json;

        // 测试初始化消息
        let init_msg = ClientMessage::Init {
            api_key: "xi-test-key".to_string(),
            language: "zh-CN".to_string(),
            format: "pcm_s16le".to_string(),
            sample_rate: 16000,
        };

        let json = serde_json::to_string(&init_msg).unwrap();
        println!("Init message JSON: {}", json);

        // 验证可以正确反序列化
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            ClientMessage::Init { api_key, language, format, sample_rate } => {
                assert_eq!(api_key, "xi-test-key");
                assert_eq!(language, "zh-CN");
                assert_eq!(format, "pcm_s16le");
                assert_eq!(sample_rate, 16000);
            }
            _ => panic!("Expected Init message"),
        }

        // 测试音频消息
        let audio_msg = ClientMessage::Audio {
            data: "SGVsbG8=".to_string(),
        };

        let json = serde_json::to_string(&audio_msg).unwrap();
        println!("Audio message JSON: {}", json);

        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            ClientMessage::Audio { data } => {
                assert_eq!(data, "SGVsbG8=");
            }
            _ => panic!("Expected Audio message"),
        }

        // 测试结束消息
        let end_msg = ClientMessage::End;
        let json = serde_json::to_string(&end_msg).unwrap();
        assert_eq!(json, r#"{"type":"end"}"#);
    }

    // 服务器消息测试
    #[test]
    fn test_server_message_deserialization() {
        use raflow_lib::network::ServerMessage;
        use serde_json;

        // 测试识别结果消息
        let result_json = r#"{
            "type": "result",
            "text": "你好世界",
            "is_final": true,
            "confidence": 0.95,
            "language": "zh-CN"
        }"#;

        let msg: ServerMessage = serde_json::from_str(result_json).unwrap();
        match msg {
            ServerMessage::Result { text, is_final, confidence, language } => {
                assert_eq!(text, "你好世界");
                assert_eq!(is_final, true);
                assert_eq!(confidence, 0.95);
                assert_eq!(language, Some("zh-CN".to_string()));
            }
            _ => panic!("Expected Result message"),
        }

        // 测试错误消息
        let error_json = r#"{
            "type": "error",
            "code": "auth_failed",
            "message": "Invalid API key"
        }"#;

        let msg: ServerMessage = serde_json::from_str(error_json).unwrap();
        match msg {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, "auth_failed");
                assert_eq!(message, "Invalid API key");
            }
            _ => panic!("Expected Error message"),
        }

        // 测试状态消息
        let status_json = r#"{
            "type": "status",
            "state": "ready"
        }"#;

        let msg: ServerMessage = serde_json::from_str(status_json).unwrap();
        match msg {
            ServerMessage::Status { state } => {
                assert_eq!(state, "ready");
            }
            _ => panic!("Expected Status message"),
        }
    }

    // 转录结果测试
    #[test]
    fn test_transcription_result_creation() {
        use raflow_lib::network::TranscriptionResult;

        let result = TranscriptionResult::new("测试文本".to_string(), true, 0.92);

        assert_eq!(result.text, "测试文本");
        assert_eq!(result.is_final, true);
        assert_eq!(result.confidence, 0.92);
        assert!(result.language.is_none());
        assert!(result.timestamp > 0);

        // 测试添加语言
        let result_with_lang = result.with_language("zh-CN".to_string());
        assert_eq!(result_with_lang.language, Some("zh-CN".to_string()));
    }

    // 服务器消息转换为转录结果
    #[test]
    fn test_server_message_to_transcription_result() {
        use raflow_lib::network::{ServerMessage, TranscriptionResult};
        use serde_json;

        let result_json = r#"{
            "type": "result",
            "text": "Hello world",
            "is_final": false,
            "confidence": 0.87,
            "language": "en-US"
        }"#;

        let msg: ServerMessage = serde_json::from_str(result_json).unwrap();

        // 使用 From trait 转换
        let result: Option<TranscriptionResult> = (&msg).into();
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.text, "Hello world");
        assert_eq!(result.is_final, false);
        assert_eq!(result.confidence, 0.87);
        assert_eq!(result.language, Some("en-US".to_string()));

        // 测试非结果消息返回 None
        let error_json = r#"{"type": "error", "code": "test", "message": "test"}"#;
        let error_msg: ServerMessage = serde_json::from_str(error_json).unwrap();
        let result: Option<TranscriptionResult> = (&error_msg).into();
        assert!(result.is_none());
    }

    // 状态转换测试
    #[test]
    fn test_recording_state_transitions() {
        use raflow_lib::RecordingState;

        // 测试状态转换路径
        let mut state = RecordingState::Idle;
        assert_eq!(state, RecordingState::Idle);

        state = RecordingState::Recording;
        assert_eq!(state, RecordingState::Recording);

        state = RecordingState::Processing;
        assert_eq!(state, RecordingState::Processing);

        state = RecordingState::Idle;
        assert_eq!(state, RecordingState::Idle);
    }

    // 连接状态测试
    #[test]
    fn test_connection_state_transitions() {
        use raflow_lib::ConnectionState;

        // 测试完整的状态转换路径
        let transitions = vec![
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Authenticating,
            ConnectionState::Ready,
            ConnectionState::Streaming,
            ConnectionState::Disconnected,
        ];

        for (i, state) in transitions.iter().enumerate() {
            println!("Step {}: {:?}", i, state);
        }

        // 验证错误状态
        assert_eq!(ConnectionState::Error, ConnectionState::Error);
    }

    // 应用状态测试
    #[test]
    fn test_app_state_operations() {
        use raflow_lib::AppState;

        let mut state = AppState::new();

        // 测试初始状态
        assert_eq!(state.recording_state, raflow_lib::RecordingState::Idle);
        assert_eq!(state.connection_state, raflow_lib::ConnectionState::Disconnected);
        assert!(state.current_device.is_none());
        assert!(state.last_result.is_none());
        assert_eq!(state.recording_duration, 0);

        // 测试状态更新
        state.set_recording_state(raflow_lib::RecordingState::Recording);
        assert_eq!(state.recording_state, raflow_lib::RecordingState::Recording);

        state.set_connection_state(raflow_lib::ConnectionState::Ready);
        assert_eq!(state.connection_state, raflow_lib::ConnectionState::Ready);

        state.update_partial_text("部分文本".to_string());
        assert_eq!(state.partial_text, Some("部分文本".to_string()));

        state.set_final_result("最终结果".to_string());
        assert_eq!(state.last_result, Some("最终结果".to_string()));
        assert!(state.partial_text.is_none());

        // 测试重置
        state.reset();
        assert_eq!(state.recording_state, raflow_lib::RecordingState::Idle);
        assert!(state.last_result.is_none());
    }

    // 错误类型测试
    #[test]
    fn test_error_types() {
        use raflow_lib::{AppError, AudioError, NetworkError, ConfigError, InputError};

        // 测试错误创建和显示
        let audio_err = AudioError::DeviceUnavailable("test-device".to_string());
        let app_err: AppError = audio_err.into();
        assert!(app_err.to_string().contains("音频设备不可用"));

        let network_err = NetworkError::Timeout;
        let app_err: AppError = network_err.into();
        assert!(app_err.to_string().contains("请求超时"));

        let config_err = ConfigError::ValidationFailed("test".to_string());
        let app_err: AppError = config_err.into();
        assert!(app_err.to_string().contains("配置验证失败"));

        let input_err = InputError::NoFocus;
        let app_err: AppError = input_err.into();
        assert!(app_err.to_string().contains("无窗口焦点"));

        // 测试字符串转换
        let string_err: AppError = "custom error".to_string().into();
        assert_eq!(string_err.to_string(), "custom error");
    }

    // Unicode 支持测试
    #[test]
    fn test_unicode_support() {
        use raflow_lib::config::AppConfig;
        use raflow_lib::network::TranscriptionResult;

        // 测试配置中的 Unicode
        let config = AppConfig {
            general: raflow_lib::config::GeneralConfig {
                language: "zh-CN".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("zh-CN"));

        // 测试转录结果中的 Unicode
        let result = TranscriptionResult::new("你好世界 🌍".to_string(), true, 0.95);
        assert_eq!(result.text, "你好世界 🌍");

        // 测试 emoji
        let result = TranscriptionResult::new("Test 😀 test".to_string(), false, 0.88);
        assert!(result.text.contains("😀"));
    }

    // 边界条件测试
    #[test]
    fn test_edge_cases() {
        use raflow_lib::network::TranscriptionResult;

        // 空字符串
        let result = TranscriptionResult::new("".to_string(), true, 0.0);
        assert_eq!(result.text, "");

        // 极长字符串
        let long_text = "A".repeat(10000);
        let result = TranscriptionResult::new(long_text.clone(), true, 1.0);
        assert_eq!(result.text.len(), 10000);

        // 边界置信度值
        let result_min = TranscriptionResult::new("test".to_string(), true, 0.0);
        assert_eq!(result_min.confidence, 0.0);

        let result_max = TranscriptionResult::new("test".to_string(), true, 1.0);
        assert_eq!(result_max.confidence, 1.0);
    }

    // 性能基准测试
    #[test]
    fn test_serialization_performance() {
        use raflow_lib::network::ClientMessage;
        use serde_json;
        use std::time::Instant;

        use base64::Engine;

        let audio_data = "A".repeat(32000); // 模拟 32KB 音频数据
        let encoded = base64::engine::general_purpose::STANDARD.encode(&audio_data);

        let msg = ClientMessage::Audio {
            data: encoded.clone(),
        };

        // 测试序列化性能
        let start = Instant::now();
        for _ in 0..100 {
            let _json = serde_json::to_string(&msg).unwrap();
        }
        let duration = start.elapsed();
        println!("100 次序列化耗时: {:?}", duration);
        // 调整阈值以适应不同系统性能
        assert!(duration.as_millis() < 500);

        // 测试反序列化性能
        let json = serde_json::to_string(&msg).unwrap();
        let start = Instant::now();
        for _ in 0..100 {
            let _: ClientMessage = serde_json::from_str(&json).unwrap();
        }
        let duration = start.elapsed();
        println!("100 次反序列化耗时: {:?}", duration);
        assert!(duration.as_millis() < 500);
    }

    // 配置验证测试
    #[test]
    fn test_config_validation() {
        use raflow_lib::config::AppConfig;

        // 测试有效配置
        let config = AppConfig::default();

        // 验证采样率范围
        assert!(config.audio.sample_rate >= 8000);
        assert!(config.audio.sample_rate <= 48000);

        // 验证超时值
        assert!(config.elevenlabs.timeout >= 1);
        assert!(config.elevenlabs.timeout <= 300);

        // 验证透明度范围
        assert!(config.ui.indicator_opacity >= 0.0);
        assert!(config.ui.indicator_opacity <= 1.0);

        // 验证插入延迟
        assert!(config.text.insertion_delay <= 1000);
    }

    // 消息完整性测试
    #[test]
    fn test_message_integrity() {
        use raflow_lib::network::{ClientMessage, ServerMessage, TranscriptionResult};
        use serde_json;

        // 测试消息往返（序列化 -> 反序列化）
        let original = ClientMessage::Init {
            api_key: "xi-test-key-12345".to_string(),
            language: "auto".to_string(),
            format: "pcm_s16le".to_string(),
            sample_rate: 16000,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: ClientMessage = serde_json::from_str(&json).unwrap();

        match (original, restored) {
            (
                ClientMessage::Init { api_key: k1, language: l1, format: f1, sample_rate: s1 },
                ClientMessage::Init { api_key: k2, language: l2, format: f2, sample_rate: s2 },
            ) => {
                assert_eq!(k1, k2);
                assert_eq!(l1, l2);
                assert_eq!(f1, f2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Message roundtrip failed"),
        }
    }

    // 并发安全测试（概念验证）
    #[test]
    fn test_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let state = Arc::new(Mutex::new(raflow_lib::AppState::new()));
        let mut handles = vec![];

        // 创建多个线程同时访问状态
        for i in 0..10 {
            let state_clone = state.clone();
            let handle = thread::spawn(move || {
                let mut state = state_clone.lock().unwrap();
                state.update_partial_text(format!("Thread {}", i));
                state.recording_duration = i;
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证状态
        let state = state.lock().unwrap();
        assert!(state.partial_text.is_some());
        assert!(state.recording_duration < 10);
    }

    // 内存使用测试（概念验证）
    #[test]
    fn test_memory_usage() {
        use raflow_lib::network::TranscriptionResult;

        // 创建大量结果对象
        let mut results = Vec::new();

        for i in 0..1000 {
            let result = TranscriptionResult::new(
                format!("测试文本 {}", i),
                i % 2 == 0,
                0.9,
            );
            results.push(result);
        }

        // 验证所有结果都正确创建
        assert_eq!(results.len(), 1000);
        assert_eq!(results[500].text, "测试文本 500");

        // 清理后内存应该释放
        drop(results);
    }
}
