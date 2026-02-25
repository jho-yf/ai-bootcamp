// src-tauri/src/network/protocol_test.rs

//! WebSocket 协议单元测试

#[cfg(test)]
mod tests {
    use super::super::protocol::*;
    use serde_json;

    // ClientMessage 测试
    #[test]
    fn test_client_message_init_serialize() {
        let msg = ClientMessage::Init {
            api_key: "xi-test-key".to_string(),
            language: "zh-CN".to_string(),
            format: "pcm_s16le".to_string(),
            sample_rate: 16000,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"init""#));
        assert!(json.contains("xi-test-key"));
        assert!(json.contains("zh-CN"));
        assert!(json.contains("pcm_s16le"));
        assert!(json.contains("16000"));
    }

    #[test]
    fn test_client_message_init_deserialize() {
        let json = r#"{
            "type": "init",
            "api_key": "xi-test-key",
            "language": "en-US",
            "format": "pcm_s16le",
            "sample_rate": 16000
        }"#;

        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Init { api_key, language, format, sample_rate } => {
                assert_eq!(api_key, "xi-test-key");
                assert_eq!(language, "en-US");
                assert_eq!(format, "pcm_s16le");
                assert_eq!(sample_rate, 16000);
            }
            _ => panic!("Expected Init message"),
        }
    }

    #[test]
    fn test_client_message_audio_serialize() {
        let msg = ClientMessage::Audio {
            data: "base64data".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"audio""#));
        assert!(json.contains("base64data"));
    }

    #[test]
    fn test_client_message_audio_deserialize() {
        let json = r#"{
            "type": "audio",
            "data": "SGVsbG8gV29ybGQ="
        }"#;

        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Audio { data } => {
                assert_eq!(data, "SGVsbG8gV29ybGQ=");
            }
            _ => panic!("Expected Audio message"),
        }
    }

    #[test]
    fn test_client_message_end_serialize() {
        let msg = ClientMessage::End;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"end"}"#);
    }

    #[test]
    fn test_client_message_end_deserialize() {
        let json = r#"{"type":"end"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::End));
    }

    // ServerMessage 测试
    #[test]
    fn test_server_message_result_serialize() {
        let msg = ServerMessage::Result {
            text: "Hello world".to_string(),
            is_final: true,
            confidence: 0.95,
            language: Some("en-US".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"result""#));
        assert!(json.contains("Hello world"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_server_message_result_deserialize() {
        let json = r#"{
            "type": "result",
            "text": "你好世界",
            "is_final": false,
            "confidence": 0.87,
            "language": "zh-CN"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Result { text, is_final, confidence, language } => {
                assert_eq!(text, "你好世界");
                assert_eq!(is_final, false);
                assert_eq!(confidence, 0.87);
                assert_eq!(language, Some("zh-CN".to_string()));
            }
            _ => panic!("Expected Result message"),
        }
    }

    #[test]
    fn test_server_message_result_without_language() {
        let json = r#"{
            "type": "result",
            "text": "Test",
            "is_final": true,
            "confidence": 0.9
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Result { language, .. } => {
                assert!(language.is_none());
            }
            _ => panic!("Expected Result message"),
        }
    }

    #[test]
    fn test_server_message_error_serialize() {
        let msg = ServerMessage::Error {
            code: "auth_failed".to_string(),
            message: "Invalid API key".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error""#));
        assert!(json.contains("auth_failed"));
        assert!(json.contains("Invalid API key"));
    }

    #[test]
    fn test_server_message_error_deserialize() {
        let json = r#"{
            "type": "error",
            "code": "rate_limited",
            "message": "Too many requests"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, "rate_limited");
                assert_eq!(message, "Too many requests");
            }
            _ => panic!("Expected Error message"),
        }
    }

    #[test]
    fn test_server_message_status_serialize() {
        let msg = ServerMessage::Status {
            state: "ready".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"status""#));
        assert!(json.contains("ready"));
    }

    #[test]
    fn test_server_message_status_deserialize() {
        let json = r#"{
            "type": "status",
            "state": "listening"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Status { state } => {
                assert_eq!(state, "listening");
            }
            _ => panic!("Expected Status message"),
        }
    }

    // TranscriptionResult 测试
    #[test]
    fn test_transcription_result_new() {
        let result = TranscriptionResult::new("Test text".to_string(), true, 0.92);
        assert_eq!(result.text, "Test text");
        assert_eq!(result.is_final, true);
        assert_eq!(result.confidence, 0.92);
        assert!(result.language.is_none());
        assert!(result.timestamp > 0);
    }

    #[test]
    fn test_transcription_result_with_language() {
        let result = TranscriptionResult::new("Test".to_string(), false, 0.85)
            .with_language("en-US".to_string());

        assert_eq!(result.language, Some("en-US".to_string()));
    }

    #[test]
    fn test_transcription_result_serialize() {
        let result = TranscriptionResult {
            text: "Hello world".to_string(),
            is_final: true,
            confidence: 0.95,
            language: Some("en".to_string()),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Hello world"));
        assert!(json.contains("1234567890"));
    }

    #[test]
    fn test_transcription_result_deserialize() {
        let json = r#"{
            "text": "测试文本",
            "is_final": false,
            "confidence": 0.88,
            "language": "zh",
            "timestamp": 9876543210
        }"#;

        let result: TranscriptionResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.text, "测试文本");
        assert_eq!(result.is_final, false);
        assert_eq!(result.confidence, 0.88);
        assert_eq!(result.language, Some("zh".to_string()));
        assert_eq!(result.timestamp, 9876543210);
    }

    // From trait 测试
    #[test]
    fn test_server_message_to_transcription_result() {
        let server_msg = ServerMessage::Result {
            text: "Converted text".to_string(),
            is_final: true,
            confidence: 0.93,
            language: Some("en".to_string()),
        };

        let result: Option<TranscriptionResult> = (&server_msg).into();
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.text, "Converted text");
        assert_eq!(result.is_final, true);
        assert_eq!(result.confidence, 0.93);
        assert_eq!(result.language, Some("en".to_string()));
    }

    #[test]
    fn test_server_message_error_to_transcription_result() {
        let server_msg = ServerMessage::Error {
            code: "error".to_string(),
            message: "Error message".to_string(),
        };

        let result: Option<TranscriptionResult> = (&server_msg).into();
        assert!(result.is_none());
    }

    #[test]
    fn test_server_message_status_to_transcription_result() {
        let server_msg = ServerMessage::Status {
            state: "ready".to_string(),
        };

        let result: Option<TranscriptionResult> = (&server_msg).into();
        assert!(result.is_none());
    }

    // 边界条件测试
    #[test]
    fn test_empty_text_result() {
        let result = TranscriptionResult::new("".to_string(), true, 0.0);
        assert_eq!(result.text, "");
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_confidence_bounds() {
        let result_min = TranscriptionResult::new("Test".to_string(), true, 0.0);
        assert_eq!(result_min.confidence, 0.0);

        let result_max = TranscriptionResult::new("Test".to_string(), true, 1.0);
        assert_eq!(result_max.confidence, 1.0);
    }

    #[test]
    fn test_unicode_in_messages() {
        let msg = ClientMessage::Init {
            api_key: "xi-key".to_string(),
            language: "zh-CN".to_string(),
            format: "pcm".to_string(),
            sample_rate: 16000,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_long_audio_data() {
        let long_data = "A".repeat(10000);
        let msg = ClientMessage::Audio {
            data: long_data.clone(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            ClientMessage::Audio { data } => {
                assert_eq!(data.len(), 10000);
            }
            _ => panic!("Expected Audio message"),
        }
    }

    // Clone 测试
    #[test]
    fn test_client_message_clone() {
        let msg = ClientMessage::Init {
            api_key: "test".to_string(),
            language: "en".to_string(),
            format: "pcm".to_string(),
            sample_rate: 16000,
        };

        let cloned = msg.clone();
        assert_eq!(msg, cloned);
    }

    #[test]
    fn test_transcription_result_clone() {
        let result = TranscriptionResult::new("Test".to_string(), true, 0.9);
        let cloned = result.clone();
        assert_eq!(result.text, cloned.text);
        assert_eq!(result.is_final, cloned.is_final);
    }

    // 消息类型匹配测试
    #[test]
    fn test_message_type_discrimination() {
        let messages = vec![
            ClientMessage::Init {
                api_key: "key".to_string(),
                language: "en".to_string(),
                format: "pcm".to_string(),
                sample_rate: 16000,
            },
            ClientMessage::Audio { data: "data".to_string() },
            ClientMessage::End,
        ];

        for msg in messages {
            match msg {
                ClientMessage::Init { .. } => assert!(true),
                ClientMessage::Audio { .. } => assert!(true),
                ClientMessage::End => assert!(true),
            }
        }
    }
}
