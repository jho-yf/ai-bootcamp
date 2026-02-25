// src-tauri/src/core/state_test.rs

//! 应用状态单元测试

#[cfg(test)]
mod tests {
    use super::super::state::*;
    use serde_json;

    // RecordingState 测试
    #[test]
    fn test_recording_state_default() {
        let state = RecordingState::default();
        assert_eq!(state, RecordingState::Idle);
    }

    #[test]
    fn test_recording_state_equality() {
        assert_eq!(RecordingState::Idle, RecordingState::Idle);
        assert_eq!(RecordingState::Recording, RecordingState::Recording);
        assert_eq!(RecordingState::Processing, RecordingState::Processing);

        assert_ne!(RecordingState::Idle, RecordingState::Recording);
        assert_ne!(RecordingState::Recording, RecordingState::Processing);
    }

    #[test]
    fn test_recording_state_clone() {
        let state = RecordingState::Recording;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_recording_state_serialize() {
        let state = RecordingState::Recording;
        let serialized = serde_json::to_string(&state).unwrap();
        assert!(serialized.contains("\"Recording\""));
    }

    #[test]
    fn test_recording_state_deserialize() {
        let json = "\"Recording\"";
        let state: RecordingState = serde_json::from_str(json).unwrap();
        assert_eq!(state, RecordingState::Recording);
    }

    // ConnectionState 测试
    #[test]
    fn test_connection_state_default() {
        let state = ConnectionState::default();
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_eq!(ConnectionState::Authenticating, ConnectionState::Authenticating);
        assert_eq!(ConnectionState::Ready, ConnectionState::Ready);
        assert_eq!(ConnectionState::Streaming, ConnectionState::Streaming);
        assert_eq!(ConnectionState::Error, ConnectionState::Error);
    }

    #[test]
    fn test_connection_state_serialize() {
        let state = ConnectionState::Ready;
        let serialized = serde_json::to_string(&state).unwrap();
        assert!(serialized.contains("\"Ready\""));
    }

    #[test]
    fn test_connection_state_deserialize() {
        let json = "\"Ready\"";
        let state: ConnectionState = serde_json::from_str(json).unwrap();
        assert_eq!(state, ConnectionState::Ready);
    }

    // AppState 测试
    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.recording_state, RecordingState::Idle);
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
        assert!(state.current_device.is_none());
        assert!(state.last_result.is_none());
        assert_eq!(state.recording_duration, 0);
        assert!(state.partial_text.is_none());
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.recording_state, RecordingState::Idle);
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_app_state_reset() {
        let mut state = AppState::default();
        state.recording_state = RecordingState::Recording;
        state.connection_state = ConnectionState::Streaming;
        state.current_device = Some("device-1".to_string());
        state.last_result = Some("result".to_string());
        state.recording_duration = 60;
        state.partial_text = Some("partial".to_string());

        state.reset();

        assert_eq!(state.recording_state, RecordingState::Idle);
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
        assert!(state.current_device.is_none());
        assert!(state.last_result.is_none());
        assert_eq!(state.recording_duration, 0);
        assert!(state.partial_text.is_none());
    }

    #[test]
    fn test_app_state_set_recording_state() {
        let mut state = AppState::default();
        state.set_recording_state(RecordingState::Recording);
        assert_eq!(state.recording_state, RecordingState::Recording);

        state.set_recording_state(RecordingState::Processing);
        assert_eq!(state.recording_state, RecordingState::Processing);
    }

    #[test]
    fn test_app_state_set_connection_state() {
        let mut state = AppState::default();
        state.set_connection_state(ConnectionState::Connecting);
        assert_eq!(state.connection_state, ConnectionState::Connecting);

        state.set_connection_state(ConnectionState::Ready);
        assert_eq!(state.connection_state, ConnectionState::Ready);
    }

    #[test]
    fn test_app_state_update_partial_text() {
        let mut state = AppState::default();
        state.update_partial_text("Hello".to_string());
        assert_eq!(state.partial_text, Some("Hello".to_string()));

        state.update_partial_text("Hello World".to_string());
        assert_eq!(state.partial_text, Some("Hello World".to_string()));
    }

    #[test]
    fn test_app_state_set_final_result() {
        let mut state = AppState::default();
        state.update_partial_text("partial".to_string());
        state.set_final_result("final result".to_string());

        assert_eq!(state.last_result, Some("final result".to_string()));
        assert!(state.partial_text.is_none());
    }

    #[test]
    fn test_app_state_clone() {
        let mut state = AppState::default();
        state.recording_state = RecordingState::Recording;
        state.current_device = Some("device".to_string());

        let cloned = state.clone();
        assert_eq!(cloned.recording_state, RecordingState::Recording);
        assert_eq!(cloned.current_device, Some("device".to_string()));
    }

    #[test]
    fn test_app_state_serialize() {
        let mut state = AppState::default();
        state.recording_state = RecordingState::Recording;
        state.current_device = Some("device-1".to_string());

        let serialized = serde_json::to_string(&state).unwrap();
        assert!(serialized.contains("\"Recording\""));
        assert!(serialized.contains("device-1"));
    }

    #[test]
    fn test_app_state_deserialize() {
        let json = r#"{
            "recording_state": "Recording",
            "connection_state": "Ready",
            "current_device": "device-1",
            "last_result": null,
            "recording_duration": 30,
            "partial_text": "partial"
        }"#;

        let state: AppState = serde_json::from_str(json).unwrap();
        assert_eq!(state.recording_state, RecordingState::Recording);
        assert_eq!(state.connection_state, ConnectionState::Ready);
        assert_eq!(state.current_device, Some("device-1".to_string()));
        assert_eq!(state.recording_duration, 30);
        assert_eq!(state.partial_text, Some("partial".to_string()));
    }

    // 状态转换测试
    #[test]
    fn test_recording_state_transitions() {
        let mut state = AppState::default();
        assert_eq!(state.recording_state, RecordingState::Idle);

        // Idle -> Recording
        state.set_recording_state(RecordingState::Recording);
        assert_eq!(state.recording_state, RecordingState::Recording);

        // Recording -> Processing
        state.set_recording_state(RecordingState::Processing);
        assert_eq!(state.recording_state, RecordingState::Processing);

        // Processing -> Idle
        state.set_recording_state(RecordingState::Idle);
        assert_eq!(state.recording_state, RecordingState::Idle);
    }

    #[test]
    fn test_connection_state_transitions() {
        let mut state = AppState::default();
        assert_eq!(state.connection_state, ConnectionState::Disconnected);

        // Disconnected -> Connecting
        state.set_connection_state(ConnectionState::Connecting);
        assert_eq!(state.connection_state, ConnectionState::Connecting);

        // Connecting -> Connected
        state.set_connection_state(ConnectionState::Connected);
        assert_eq!(state.connection_state, ConnectionState::Connected);

        // Connected -> Authenticating
        state.set_connection_state(ConnectionState::Authenticating);
        assert_eq!(state.connection_state, ConnectionState::Authenticating);

        // Authenticating -> Ready
        state.set_connection_state(ConnectionState::Ready);
        assert_eq!(state.connection_state, ConnectionState::Ready);

        // Ready -> Streaming
        state.set_connection_state(ConnectionState::Streaming);
        assert_eq!(state.connection_state, ConnectionState::Streaming);

        // Streaming -> Disconnected
        state.set_connection_state(ConnectionState::Disconnected);
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
    }

    // 边界条件测试
    #[test]
    fn test_app_state_with_empty_string() {
        let mut state = AppState::default();
        state.current_device = Some("".to_string());
        assert_eq!(state.current_device, Some("".to_string()));

        state.last_result = Some("".to_string());
        assert_eq!(state.last_result, Some("".to_string()));
    }

    #[test]
    fn test_app_state_with_unicode() {
        let mut state = AppState::default();
        state.last_result = Some("你好世界".to_string());
        state.partial_text = Some("テスト".to_string());

        assert_eq!(state.last_result, Some("你好世界".to_string()));
        assert_eq!(state.partial_text, Some("テスト".to_string()));
    }

    #[test]
    fn test_recording_duration_increment() {
        let mut state = AppState::default();
        assert_eq!(state.recording_duration, 0);

        state.recording_duration = 1;
        assert_eq!(state.recording_duration, 1);

        state.recording_duration = 3600; // 1 hour
        assert_eq!(state.recording_duration, 3600);
    }
}
