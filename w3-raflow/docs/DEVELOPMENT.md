# RaFlow 开发者文档

## 目录

- [架构设计](#架构设计)
- [开发环境设置](#开发环境设置)
- [代码规范](#代码规范)
- [API 文档](#api-文档)
- [测试指南](#测试指南)
- [构建与打包](#构建与打包)
- [故障排查](#故障排查)

---

## 架构设计

### 系统架构

RaFlow 采用模块化设计，核心分为前端和后端两部分：

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   UI     │  │  Stores  │  │   API    │  │Components│  │
│  │ (React)  │  │(Zustand)│  │ (Tauri)  │  │  (TSX)   │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                    Tauri IPC Bridge
                            │
┌─────────────────────────────────────────────────────────────┐
│                       Backend (Rust)                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Config  │  │  Audio   │  │ Network  │  │  Input   │  │
│  │  Module  │  │  Module  │  │  Module  │  │  Module  │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                │
│  │   Core   │  │  Hotkey  │  │  Tray    │                │
│  │  Module  │  │  Module  │  │  Module  │                │
│  └──────────┘  └──────────┘  └──────────┘                │
└─────────────────────────────────────────────────────────────┘
```

### 模块说明

#### 后端模块 (Rust)

| 模块 | 路径 | 职责 |
|------|------|------|
| `core` | `src-tauri/src/core/` | 核心应用状态、错误类型定义 |
| `audio` | `src-tauri/src/audio/` | 音频采集、设备管理、缓冲池 |
| `network` | `src-tauri/src/network/` | WebSocket 通信、协议定义、网络优化 |
| `input` | `src-tauri/src/input/` | 键盘输入模拟、剪贴板操作 |
| `config` | `src-tauri/src/config/` | 配置管理、持久化存储 |
| `hotkey` | `src-tauri/src/hotkey/` | 全局热键注册与处理 |
| `tray` | `src-tauri/src/tray/` | 系统托盘图标与菜单 |
| `updater` | `src-tauri/src/updater/` | 自动更新功能 |

#### 前端模块 (TypeScript/React)

| 模块 | 路径 | 职责 |
|------|------|------|
| `stores` | `src/stores/` | Zustand 状态管理 |
| `api` | `src/api/` | Tauri IPC 调用封装 |
| `components` | `src/components/` | UI 组件 |

### 数据流

1. **录音流程**:
   - 用户按下热键 → `HotkeyModule` 触发
   - `AudioService` 开始录音 → 音频数据通过 WebSocket 发送到 ElevenLabs
   - 接收转录结果 → 通过 IPC 发送到前端
   - 前端更新 UI → `InputService` 模拟键盘输入

2. **配置流程**:
   - 前端调用 `configApi.saveConfig()`
   - Rust 端保存到 TOML 文件
   - 应用状态自动更新

---

## 开发环境设置

### 前置要求

- **Rust**: 1.85+ (推荐使用 rustup 安装)
- **Node.js**: 20+
- **pnpm**: 9+ (推荐) 或 npm

### 安装依赖

```bash
# 克隆仓库
git clone https://github.com/ai-bootcamp/raflow.git
cd raflow

# 安装 Rust 依赖（工作区自动管理）
cargo fetch

# 安装前端依赖
pnpm install
```

### 开发模式

```bash
# 启动开发服务器（热重载）
pnpm tauri dev
```

### IDE 配置

推荐使用 VS Code，安装以下扩展：

- `rust-analyzer`: Rust 语言支持
- `ESLint`: TypeScript/JavaScript 代码检查
- `Prettier`: 代码格式化
- `Tailwind CSS IntelliSense`: CSS 类名智能提示

---

## 代码规范

### Rust 代码规范

1. **使用 `?` 运算符处理错误**，避免使用 `unwrap()` 或 `expect()`

2. **优先使用 `mpsc channel` 而非 `shared state`**

3. **对于很少变动的数据（如配置），优先考虑 `ArcSwap`**

4. **需要 `HashMap` 时，优先使用 `DashMap`**

5. **禁止使用 `unsafe` 代码**

6. **使用 Rust 最新版本的 `async trait` 支持**

### TypeScript 代码规范

1. **使用函数式组件 + Hooks**

2. **使用 Zustand 进行状态管理**

3. **组件命名使用 PascalCase**

4. **API 函数使用 camelCase**

5. **导出类型定义以供复用**

### Git 提交规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型 (type)**:
- `feat`: 新功能
- `fix`: 修复 bug
- `refactor`: 重构
- `docs`: 文档更新
- `test`: 测试相关
- `chore`: 构建/工具链相关

**示例**:
```
feat(audio): add audio buffer pool for memory optimization

Implement AudioBufferPool to reduce memory allocation overhead
by reusing audio buffers.

Closes #123
```

---

## API 文档

### Tauri Commands (Rust → Frontend)

#### 配置命令

| 命令 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `get_config` | - | `AppConfig` | 获取当前配置 |
| `save_config` | `config: AppConfig` | `()` | 保存配置 |
| `reset_config` | - | `AppConfig` | 重置为默认配置 |
| `get_config_schema` | - | `Record<string, unknown>` | 获取配置 JSON Schema |

#### 音频命令

| 命令 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `get_audio_devices` | - | `AudioDevice[]` | 获取音频设备列表 |
| `start_recording` | `deviceId?: string` | `()` | 开始录音 |
| `stop_recording` | - | `()` | 停止录音 |
| `test_microphone` | `deviceId: string` | `boolean` | 测试麦克风可用性 |
| `get_recording_state` | - | `boolean` | 获取录音状态 |

#### 热键命令

| 命令 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `register_hotkey` | `modifiers: string[], key: string` | `()` | 注册热键 |
| `unregister_hotkey` | - | `()` | 注销热键 |
| `test_hotkey` | - | `boolean` | 测试热键是否可用 |

#### 更新命令

| 命令 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `check_for_updates` | `updateUrl: string` | `UpdateInfo` | 检查更新 |
| `get_update_status` | - | `UpdateStatus` | 获取更新状态 |

### 事件 (Rust → Frontend)

| 事件 | 负载 | 描述 |
|------|------|------|
| `recording-started` | - | 录音开始 |
| `recording-stopped` | `text?: string` | 录音停止 |
| `transcription-result` | `TranscriptionResult` | 最终转录结果 |
| `partial_transcription` | `string` | 部分转录结果 |
| `error` | `string` | 错误信息 |
| `connection-state-changed` | `boolean` | 连接状态变化 |
| `update-status` | `UpdateStatus` | 更新状态变化 |

---

## 测试指南

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试（排除集成测试）
cargo test --lib

# 运行集成测试
cargo test --test integration_test

# 运行特定测试
cargo test test_audio_buffer_pool
```

### 测试覆盖

- **单元测试**: 位于各模块的 `tests` 模块中
- **集成测试**: 位于 `src-tauri/tests/` 目录

### 添加新测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Arrange
        let input = "test";

        // Act
        let result = process(input);

        // Assert
        assert_eq!(result, "expected");
    }
}
```

---

## 构建与打包

### 开发构建

```bash
pnpm tauri build --debug
```

### 生产构建

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：

| 平台 | 产物 |
|------|------|
| Linux | `.deb`, `.AppImage` |
| macOS | `.dmg`, `.app` |
| Windows | `.msi`, `.exe` |

### 签名配置

在 `src-tauri/tauri.conf.json` 中配置签名信息：

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name"
    },
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT"
    }
  }
}
```

---

## 故障排查

### 常见问题

#### 1. 音频设备不可用

**症状**: `AudioError::DeviceUnavailable`

**解决方案**:
- 检查系统权限设置
- 确认麦克风未被其他应用占用
- 尝试更换音频设备

#### 2. 网络连接失败

**症状**: `NetworkError::ConnectionFailed`

**解决方案**:
- 检查网络连接
- 验证 ElevenLabs API Key
- 检查防火墙设置

#### 3. 热键无法注册

**症状**: 热键按下无响应

**解决方案**:
- 确认热键组合未被其他应用占用
- 尝试更改热键组合
- 检查系统辅助功能权限 (macOS)

### 调试模式

```bash
# 启用详细日志
RUST_LOG=debug pnpm tauri dev

# 查看 Tauri 日志
tail -f ~/Library/Logs/com.ai-bootcamp.raflow/*.log  # macOS
tail -f ~/.local/share/com.ai-bootcamp.raflow/*.log   # Linux
type %LOCALAPPDATA%\com.ai-bootcamp.raflow\logs\*.log # Windows
```

---

## 贡献指南

欢迎提交 Pull Request！请确保：

1. 代码通过所有测试 (`cargo test`)
2. 代码符合项目规范
3. 添加必要的测试和文档
4. 提交信息遵循规范

---

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件
