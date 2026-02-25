# RaFlow 语音输入工具

<p align="center">
  <strong>基于 ElevenLabs Scribe v2 的跨平台实时语音输入工具</strong>
</p>

<p align="center">
  <a href="https://github.com/ai-bootcamp/raflow/releases"><img src="https://img.shields.io/github/v/release/ai-bootcamp/raflow" alt="Release"></a>
  <a href="https://github.com/ai-bootcamp/raflow/blob/main/LICENSE"><img src="https://img.shields.io/github/license/ai-bootcamp/raflow" alt="License"></a>
  <a href="https://github.com/ai-bootcamp/raflow/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ai-bootcamp/raflow/ci.yml" alt="CI"></a>
</p>

## ✨ 功能特性

- 🎤 **实时语音转文字** - 低延迟转录（<150ms）
- 🌍 **多语言支持** - 支持中文、英语、日语等多种语言
- ⌨️ **智能文本插入** - 自动将识别文本插入到光标位置
- 🎯 **全局快捷键** - 系统级快捷键触发，默认 `Ctrl+Shift+O`
- 🔊 **音频增强** - 内置回声消除、噪声抑制、自动增益
- 📋 **剪贴板备份** - 键盘输入失败时自动复制到剪贴板
- 🪟 **跨平台支持** - Windows、macOS、Linux

## 🚀 快速开始

### 安装

从 [Releases](https://github.com/ai-bootcamp/raflow/releases) 页面下载适合您操作系统的安装包。

#### Windows
- **MSI 安装包** (推荐): 双击安装
- **ZIP 便携版**: 解压后直接运行 `RaFlow.exe`

#### macOS
- **DMG 镜像**: 打开后拖拽到 Applications 文件夹

#### Linux
- **AppImage**: 添加执行权限后直接运行
  ```bash
  chmod +x RaFlow-x86_64.AppImage
  ./RaFlow-x86_64.AppImage
  ```
- **deb 包**: 使用 apt 安装
  ```bash
  sudo dpkg -i raflow_*.deb
  ```

### 配置

1. **获取 API Key**
   - 访问 [ElevenLabs](https://elevenlabs.io)
   - 注册账号并创建 API Key
   - 复制 API Key（格式：`xi-xxxxx`）

2. **打开设置**
   - 点击系统托盘图标
   - 选择"设置..."
   - 或按 `Ctrl+,` 打开

3. **输入 API Key**
   - 在"常规"标签页中粘贴 API Key
   - 选择识别语言（默认自动检测）
   - 点击"保存设置"

### 使用

1. **开始录音**
   - 按下快捷键 `Ctrl+Shift+O`
   - 看到状态指示器显示"录音中"

2. **说话**
   - 应用会实时显示识别的文本
   - 说话清晰，语速适中

3. **停止录音**
   - 再次按下快捷键 `Ctrl+Shift+O`
   - 识别结果会自动插入到光标位置

## ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+O` | 开始/停止录音 |
| `Ctrl+,` | 打开设置 |
| `Escape` | 关闭设置窗口 |

## 🔧 配置选项

### 常规设置
- **API Key**: ElevenLabs API 密钥
- **识别语言**: 选择识别语言（自动检测/中文/英语/日语等）
- **文本插入方式**: 自动选择/仅键盘/仅剪贴板

### 音频设置
- **音频输入设备**: 选择麦克风设备
- **回声消除**: 减少回声干扰
- **噪声抑制**: 降低背景噪音
- **自动增益**: 平衡音量大小

### 快捷键设置
- **全局快捷键**: 自定义录音快捷键
- 点击快捷键输入框，然后按下想要设置的组合键
- 支持 Ctrl、Alt、Shift、Super (Win/Cmd) 修饰键组合

## ❓ 常见问题

### 无法录音？

1. 检查麦克风权限
   - **Windows**: 设置 > 隐私 > 麦克风
   - **macOS**: 系统偏好设置 > 安全性与隐私 > 隐私 > 麦克风
   - **Linux**: 检查 PulseAudio/ALSA 设置

2. 测试麦克风
   - 打开设置 > 音频
   - 点击"测试麦克风"

### 识别不准确？

1. 确保网络连接正常
2. 选择正确的识别语言
3. 在安静环境中使用
4. 说话清晰，语速适中

### 文本没有插入？

1. 检查目标应用是否支持文本输入
2. 确保光标在可输入位置
3. 尝试切换到"仅剪贴板"模式
4. 检查剪贴板内容

### 快捷键不工作？

1. 检查快捷键是否与其他应用冲突
2. 确认快捷键已启用（设置 > 快捷键）
3. 尝试重新注册快捷键

## 📦 开发

### 环境要求

- **Rust**: 1.82+
- **Node.js**: 18.x 或 20.x
- **系统依赖**:
  - Windows: Visual Studio Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: `build-essential`, `libwebkit2gtk-4.0-dev`

### 构建

```bash
# 克隆仓库
git clone https://github.com/ai-bootcamp/raflow.git
cd raflow

# 安装依赖
npm install

# 开发模式
npm run tauri:dev

# 构建生产版本
npm run tauri:build
```

### 运行测试

```bash
# Rust 单元测试
cargo test

# 前端测试
npm test

# 完整测试
npm run test
```

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [ElevenLabs](https://elevenlabs.io) - 提供语音识别 API
- [Tauri](https://tauri.app) - 跨平台应用框架
- [cpal](https://github.com/RustAudio/cpal) - 音频捕获库

## 📮 反馈与支持

- [GitHub Issues](https://github.com/ai-bootcamp/raflow/issues) - 报告问题
- [GitHub Discussions](https://github.com/ai-bootcamp/raflow/discussions) - 讨论交流

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/ai-bootcamp">AI Bootcamp Team</a>
</p>
