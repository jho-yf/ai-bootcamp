// src-tauri/src/core/error_test.rs

//! 错误类型单元测试

#[cfg(test)]
mod tests {
    use super::super::error::*;
    use std::io;

    // AudioError 测试
    #[test]
    fn test_audio_error_display() {
        let err = AudioError::DeviceUnavailable("test".to_string());
        assert_eq!(err.to_string(), "音频设备不可用: test");

        let err = AudioError::InitFailed("test".to_string());
        assert_eq!(err.to_string(), "设备初始化失败: test");

        let err = AudioError::RecordingFailed("test".to_string());
        assert_eq!(err.to_string(), "录音失败: test");

        let err = AudioError::UnsupportedFormat("pcm".to_string());
        assert_eq!(err.to_string(), "音频格式不支持: pcm");

        let err = AudioError::StreamError("test".to_string());
        assert_eq!(err.to_string(), "音频流创建失败: test");
    }

    // NetworkError 测试
    #[test]
    fn test_network_error_display() {
        let err = NetworkError::ConnectionFailed("test".to_string());
        assert_eq!(err.to_string(), "连接失败: test");

        let err = NetworkError::AuthFailed("test".to_string());
        assert_eq!(err.to_string(), "认证失败: test");

        let err = NetworkError::SendFailed("test".to_string());
        assert_eq!(err.to_string(), "发送失败: test");

        let err = NetworkError::ReceiveFailed("test".to_string());
        assert_eq!(err.to_string(), "接收失败: test");

        let err = NetworkError::Timeout;
        assert_eq!(err.to_string(), "请求超时");

        let err = NetworkError::ApiError {
            code: "401".to_string(),
            message: "Unauthorized".to_string(),
        };
        assert_eq!(err.to_string(), "API错误: 401 - Unauthorized");
    }

    // ConfigError 测试
    #[test]
    fn test_config_error_display() {
        let err = ConfigError::Corrupted("test".to_string());
        assert_eq!(err.to_string(), "配置文件损坏: test");

        let err = ConfigError::DirectoryInaccessible("/path".to_string());
        assert_eq!(err.to_string(), "配置目录不可访问: /path");

        let err = ConfigError::ValidationFailed("test".to_string());
        assert_eq!(err.to_string(), "配置验证失败: test");

        let err = ConfigError::SaveFailed("test".to_string());
        assert_eq!(err.to_string(), "配置保存失败: test");

        let err = ConfigError::LoadFailed("test".to_string());
        assert_eq!(err.to_string(), "配置加载失败: test");
    }

    // InputError 测试
    #[test]
    fn test_input_error_display() {
        let err = InputError::KeyboardFailed("test".to_string());
        assert_eq!(err.to_string(), "键盘输入失败: test");

        let err = InputError::ClipboardError("test".to_string());
        assert_eq!(err.to_string(), "剪贴板错误: test");

        let err = InputError::HotkeyRegisterFailed("test".to_string());
        assert_eq!(err.to_string(), "热键注册失败: test");

        let err = InputError::NoFocus;
        assert_eq!(err.to_string(), "无窗口焦点");

        let err = InputError::Other("test".to_string());
        assert_eq!(err.to_string(), "test");
    }

    // AppError 转换测试
    #[test]
    fn test_app_error_from_audio_error() {
        let audio_err = AudioError::DeviceUnavailable("test".to_string());
        let app_err: AppError = audio_err.into();
        assert!(matches!(app_err, AppError::Audio(_)));
        assert_eq!(app_err.to_string(), "音频错误: 音频设备不可用: test");
    }

    #[test]
    fn test_app_error_from_network_error() {
        let network_err = NetworkError::Timeout;
        let app_err: AppError = network_err.into();
        assert!(matches!(app_err, AppError::Network(_)));
        assert_eq!(app_err.to_string(), "网络错误: 请求超时");
    }

    #[test]
    fn test_app_error_from_config_error() {
        let config_err = ConfigError::ValidationFailed("test".to_string());
        let app_err: AppError = config_err.into();
        assert!(matches!(app_err, AppError::Config(_)));
        assert_eq!(app_err.to_string(), "配置错误: 配置验证失败: test");
    }

    #[test]
    fn test_app_error_from_input_error() {
        let input_err = InputError::NoFocus;
        let app_err: AppError = input_err.into();
        assert!(matches!(app_err, AppError::Input(_)));
        assert_eq!(app_err.to_string(), "输入错误: 无窗口焦点");
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
        assert!(app_err.to_string().contains("IO错误"));
    }

    #[test]
    fn test_app_error_from_string() {
        let s = "custom error".to_string();
        let app_err: AppError = s.clone().into();
        assert!(matches!(app_err, AppError::Other(_)));
        assert_eq!(app_err.to_string(), "custom error");
    }

    #[test]
    fn test_app_error_from_str() {
        let s = "custom error";
        let app_err: AppError = s.into();
        assert!(matches!(app_err, AppError::Other(_)));
        assert_eq!(app_err.to_string(), "custom error");
    }

    // Result 类型别名测试
    #[test]
    fn test_result_type_alias() {
        let result: Result<()> = Ok(());
        assert!(result.is_ok());

        let result: Result<()> = Err(AppError::Other("error".to_string()));
        assert!(result.is_err());
    }

    // 错误链测试
    #[test]
    fn test_error_chain() {
        let audio_err = AudioError::DeviceUnavailable("device".to_string());
        let app_err: AppError = audio_err.into();

        // 验证错误来源
        match app_err {
            AppError::Audio(inner) => {
                assert!(matches!(inner, AudioError::DeviceUnavailable(_)));
            }
            _ => panic!("Expected Audio error variant"),
        }
    }
}
