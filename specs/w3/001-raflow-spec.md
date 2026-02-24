# ElevenLabs Scribe v2 Realtime API 探索与 Wispr Flow 类似工具技术方案

## 一、项目背景与目标

在当今快速发展的AI时代，语音转文字技术已经成为提升工作效率的关键工具。Wispr Flow 作为一款创新的语音输入应用，能够在任意应用程序中实现即时语音转文字，其核心优势在于支持100多种语言、自动纠正拼写语法错误，以及能够在任何文本输入场景中使用。本项目旨在基于 ElevenLabs 最新发布的 Scribe v2 Realtime API，结合 Tauri 2 框架，构建一款类似的语音输入工具。

ElevenLabs 于2025年正式推出 Scribe v2 Realtime，这是一款专为强实时性需求场景深度优化的超低延迟实时语音识别模型。该模型采用以流式处理为核心的架构设计，原生支持 PCM、μ-law 等主流音频编码格式，并集成预测式转写、精准语音活动检测（VAD）、上下文感知记忆机制及专业术语自适应识别等多项关键技术。在权威多语言语音识别基准 FLEURS 上，其整体准确率高达93.5%，大幅 Gemini Flash超越 Google 2.5、OpenAI GPT-4o Mini 等同类竞品。Scribe v2 Realtime 实现语音输入至文字输出仅约150毫秒的端到端延迟，真正达成“所言即所得”的极致体验。

本技术方案将详细阐述如何利用 Tauri 2 构建一款常驻系统托盘的语音输入应用，实现全局热键触发、实时语音转写、文本自动插入目标应用光标位置等核心功能。

## 二、ElevenLabs Scribe v2 Realtime API 分析

### 2.1 API 核心特性与技术规格

ElevenLabs Scribe v2 Realtime API 代表了当前业界领先的实时语音识别技术，其核心技术规格和特性值得深入了解。该API专为需要毫秒级响应的实时应用场景设计，包括智能语音交互、会议纪要生成、直播实时字幕、语音输入工具等。

在技术架构层面，Scribe v2 Realtime 采用流式处理模式，这与传统的批量转录API有本质区别。流式处理允许在用户说话的同时实时返回识别结果，而非等待用户说完后再进行处理。这种架构对于语音输入工具至关重要，因为用户期望的是边说边看到文字出现在屏幕上。API 原生支持 PCM 和 μ-law 音频编码格式，这两种格式都是实时通信中的标准选择，能够在保证音质的同时最小化数据传输延迟。

Scribe v2 Realtime 的另一项核心技术优势是其“负延迟预测”能力。该技术能够提前预判用户即将说出的下一个词及标点符号，从而在保持高准确率的同时进一步降低感知延迟。此外，模型还具备动态语言识别功能，能够在单次对话中无缝识别并自动切换语种，这对于多语言用户群体来说极为实用。模型已覆盖超过90种语言，涵盖英语、法语、西班牙语等全球主流语种，能够完美适配国际化协作与多语种混合交流场景。

在准确率方面，Scribe v2 Realtime 在英语上的词错误率（WER）低至约5%，对印地语等全球超过90种语言的识别准确率均控制在10%以内。该模型还支持关键词引导式转录（Keyterm Prompting）功能，用户最多可预设100个专业术语、品牌名称或技术词汇，模型将结合语境智能判断并精准还原这些关键表达，大幅提升垂直领域文本质量。

### 2.2 WebSocket 连接与通信协议

ElevenLabs Scribe v2 Realtime API 采用 WebSocket 协议进行实时双向通信，这是实现低延迟语音转写的理想选择。与传统的 HTTP 请求-响应模式不同，WebSocket 允许客户端与服务器建立持久的全双工连接，双方可以随时主动传送数据，规避了 HTTP 轮询所导致的延迟以及资源开销。

API 的 WebSocket 端点通常为 `wss://api.elevenlabs.io/v1/speech-to-text/stream`，具体的端点URL可能因版本和配置而有所不同。连接建立时需要通过 HTTP 握手（状态码101切换协议），其后升级为二进制帧传输，极大地削减了通信开销。WebSocket 内置心跳机制（Ping/Pong 帧）以维持连接的活性，并且支持通过子协议扩展功能。在安全方面，API 强制使用 TLS 加密（wss://），确保数据传输的安全性。

通信协议方面，客户端需要首先发送认证信息，通常包括 API Key（xi-api-key）。认证完成后，客户端开始流式传输音频数据。音频数据可以采用原始 PCM 格式或经过编码的格式，具体取决于 API 的配置要求。服务器端则实时返回识别结果，结果通常以 JSON 格式包含识别文本、置信度、是否最终结果（is_final）等字段。

对于 TypeScript 实现，可以使用原生的 WebSocket API 或流行的 ws 库。在 Node.js 环境中，ws 库提供了简洁而强大的 WebSocket 客户端实现，支持自动重连、心跳检测等企业级功能。以下是一个简化的连接示例：

```typescript
import WebSocket from 'ws';

const apiKey = process.env.ELEVENLABS_API_KEY;
const wsUrl = 'wss://api.elevenlabs.io/v1/speech-to-text/stream';

const ws = new WebSocket(wsUrl, {
  headers: {
    'xi-api-key': apiKey
  }
});

ws.on('open', () => {
  console.log('WebSocket connected');
});

ws.on('message', (data) => {
  const result = JSON.parse(data.toString());
  if (result.is_final) {
    console.log('Final transcription:', result.text);
  }
});

ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});
```

### 2.3 音频数据格式与传输

音频数据的格式和质量直接影响识别结果的准确性。ElevenLabs Scribe v2 Realtime API 对音频格式有明确的要求，理解这些要求对于实现高质量的语音转写至关重要。

在采样率方面，API 通常支持16kHz 和 44.1kHz 两种采样率。对于语音转写任务，16kHz 通常是足够的选择，因为语音的主要频率范围在8kHz以下，使用更高的采样率不会显著提升识别准确率，反而会增加数据传输量和处理负担。然而，如果需要保留更高质量的音频用于其他用途，可以选择44.1kHz。

在声道方面，API 要求单声道（mono）音频。这意味着如果采集的是立体声数据，需要在进行流传输前进行声道合并。音频位深度通常为16位 PCM，这是语音处理的标准格式，能够提供足够的动态范围。

在传输层面，音频数据可以以原始二进制格式或 Base64 编码的 JSON 格式发送。使用二进制格式传输效率更高，但需要确保 WebSocket 连接配置为支持二进制数据。Base64 编码则更加人类可读和调试友好，但在编码过程中会产生约33%的额外开销。

以下是一个使用 Web Audio API 捕获麦克风音频并转换为所需格式的 TypeScript 示例：

```typescript
async function startAudioCapture(
  onAudioChunk: (chunk: ArrayBuffer) => void
): Promise<void> {
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: {
      sampleRate: 16000,
      channelCount: 1,
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true
    }
  });

  const audioContext = new AudioContext({ sampleRate: 16000 });
  const source = audioContext.createMediaStreamSource(stream);
  const processor = audioContext.createScriptProcessor(4096, 1, 1);

  processor.onaudioprocess = (event) => {
    const inputBuffer = event.inputBuffer;
    const channelData = inputBuffer.getChannelData(0);

    // 转换为16位 PCM
    const pcmData = new Int16Array(channelData.length);
    for (let i = 0; i < channelData.length; i++) {
      pcmData[i] = Math.max(-1, Math.min(1, channelData[i])) * 0x7FFF;
    }

    onAudioChunk(pcmData.buffer);
  };

  source.connect(processor);
  processor.connect(audioContext.destination);
}
```

## 三、Wispr Flow 竞品功能分析

### 3.1 核心功能特性

Wispr Flow 作为当前市场上最先进的语音输入工具之一，其功能特性和用户体验设计值得深入分析。该应用的核心价值主张是“降低文字输入门槛”，解决打字慢、写作累、思维与输出速度不同步的问题。

语音输入加速是 Wispr Flow 最基础也是最重要的功能。该应用声称能够帮助用户实现比传统打字快3倍的输入速度。这一成就的实现依赖于多项技术的综合优化：高质量的语音识别算法确保识别结果的准确性，低延迟的端到端处理确保用户无需等待，智能的标点和格式处理减少后期编辑工作量。对于需要大量文字输入的场景，如撰写邮件、编写报告、记录会议纪要等，这种加速效果尤为明显。

自动编辑与上下文感知是 Wispr Flow 区别于基础语音识别的关键能力。与传统语音转文字仅做简单的语音到文本映射不同，Wispr Flow 能够自动纠正拼写和语法错误，根据上下文调整文本，确保输出自然流畅。这种能力基于大规模语言模型的上下文理解能力，使得转写结果不仅准确，而且符合目标语言的语法规范和表达习惯。

多语言支持是全球化时代的必备功能。Wispr Flow 支持100多种语言，并提供“自动检测”语言功能，用户无需手动切换即可应对多语言混合的交流场景。这对于国际商务沟通、多语言内容创作等场景非常有价值。

低音量识别（耳语模式）是 Wispr Flow 的一项贴心设计。并非所有场景都适合大声说话，在图书馆、会议室或嘈杂环境中，用户可能需要“轻声细语”地使用语音输入。Wispr Flow 针对低音量语音进行了专门优化，确保在小声说话时仍能保持高准确率。

AI 命令模式将语音输入提升到一个新的层次。用户不仅可以通过语音输入文字，还可以通过语音指令直接操作文档或查询信息。例如，用户可以说“把这段话改成更正式的语气”或“查询今天的天气”，应用能够理解并执行这些指令。

### 3.2 用户交互设计

Wispr Flow 在用户交互设计方面有许多值得借鉴的创新点。理解这些设计理念对于实现类似的工具非常重要。

在激活方式上，Wispr Flow 提供了多种触发语音输入的方式，包括按住特定按键（如 FN 键）或使用 FN+空格组合键实现免手动模式。这种设计考虑到不同用户的使用习惯和场景需求。对于需要频繁切换语音输入和键盘输入的用户，可变触发方式能够显著提升使用体验。

在视觉反馈方面，应用在录音状态下会显示明确的视觉指示，通常是一个浮动的小窗口，显示“Listening..."或波形动画，告知用户当前正处于语音输入状态。这种反馈对于用户信心建立非常重要，避免用户不确定是否正在录音。

在设置流程方面，Wispr Flow 在首次使用时引导用户测试麦克风、确认功能键可用性，这种初始化流程确保用户在正式使用前一切正常。同时，应用提供14天免费试用，降低用户的尝试门槛。

### 3.3 技术实现启示

从 Wispr Flow 的功能特性分析中，我们可以提炼出实现类似工具的关键技术需求。

首先是高质量的语音识别能力。这是语音输入工具的核心，没有高准确率的语音识别，一切用户体验都是空谈。ElevenLabs Scribe v2 Realtime 正是为了满足这一需求而设计，其93.5%的准确率和150毫秒的延迟使其成为极佳的选择。

其次是低延迟的端到端处理。用户期望“所说即所得”，任何明显的延迟都会破坏使用体验。这要求从音频采集、传输、识别到文本插入的整个链路都要进行延迟优化。

第三是文本后处理能力。原始语音识别结果通常需要进一步处理才能成为可用的文本。这包括标点符号添加、大小写规范化、语法修正、重复词过滤等。实现这些功能可能需要借助大型语言模型的能力。

第四是跨应用集成能力。语音输入工具需要能够与任意文本输入应用配合工作，这意味着需要实现系统级的键盘模拟或剪贴板操作功能。

第五是状态管理与用户反馈。清晰的状态指示（空闲、录音中、转写中）以及错误处理对用户体验至关重要。

## 四、Tauri 2 技术架构设计

### 4.1 架构概述与技术选型

本项目采用 Tauri 2 作为应用框架，这是一个明智的技术选择。Tauri 是一个使用 Rust 构建跨平台桌面应用的新一代框架，它巧妙地结合了 Rust 的高性能后端和 Web 前端技术，提供了体积小、运行快、安全性高的桌面应用解决方案。

与 Electron 相比，Tauri 具有显著的优势。Electron 通过捆绑 Node.js 和 Chromium 实现了跨平台 GUI，但代价高昂：每个应用都包含一个完整的浏览器实例，内存动辄300MB以上，应用打包后体积超过100MB。相比之下，Tauri 开发的应用前端使用操作系统的 WebView，后端使用 Rust，理论上性能更优，打包后的体积仅约4MB左右。这种轻量级特性对于一个需要常驻后台的语音输入工具尤为重要。

Tauri 2 在架构上进行了一系列优化，引入了更加灵活的插件系统和更好的安全沙箱。对于本项目来说，我们将重点使用以下插件和能力：全局快捷键注册（tauri-plugin-global-shortcut）、系统托盘（tray-icon）、窗口管理等。

项目的整体架构采用“厚后端”模式，因为浏览器沙箱（前端子应用）无法轻易地在其他应用中输入文字或可靠地捕获全局热键。因此，最关键的逻辑（音频捕获、WebSocket 通信、文本插入、剪贴板操作）都将放在 Rust 后端实现。前端（React/TypeScript）主要负责设置界面、状态显示等轻量级任务。

### 4.2 Rust 后端组件设计

Rust 后端是整个应用的核心，需要实现音频捕获、WebSocket 通信、文本插入、剪贴板管理等功能。以下是各组件的详细设计。

音频捕获组件使用 cpal 库实现跨平台音频输入。cpal 是一个纯 Rust 的跨平台音频库，提供了统一的 API 来枚举音频设备、打开输入流、读取音频数据。在初始化时，后端需要枚举系统中可用的麦克风设备，让用户选择或自动选择默认设备。然后打开输入流，以指定的采样率（16kHz）和声道数（单声道）捕获音频数据。捕获的音频数据会被实时推送到 WebSocket 连接。

```rust
// Cargo.toml 中的依赖配置
[dependencies]
tauri = { version = "2.0", features = ["tray-icon", "image-png"] }
tauri-plugin-global-shortcut = "2.0"
cpal = "0.15"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
enigo = "0.1"
arboard = "3.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.21"
```

WebSocket 客户端组件使用 tokio-tungstenite 实现异步 WebSocket 通信。该组件负责管理与 ElevenLabs API 的连接，包括连接建立、认证、心跳检测、音频数据发送、识别结果接收等功能。在连接建立后，组件需要发送初始配置消息，包括 API Key、音频格式参数等。音频数据通过 Base64 编码后作为 JSON 消息发送。识别结果从服务器接收后，通过 Tauri 的事件系统转发给前端或直接触发文本插入逻辑。

文本插入组件使用 enigo 库实现跨平台键盘模拟。enigo 支持在 Linux（X11、Wayland）、macOS 和 Windows 上模拟键盘输入。当接收到最终的识别结果时，后端首先尝试将文本插入到当前活动窗口的光标位置。具体实现是调用 `enigo.text()` 方法输入文本。如果文本输入成功，则任务完成。如果失败（例如当前活动窗口不是文本输入字段），则触发备用方案。

剪贴板备用方案使用 arboard 库实现剪贴板操作。当键盘模拟失败时，后端将识别结果写入系统剪贴板，然后通过系统通知告知用户“转写文本已复制到剪贴板”。这种设计确保了即使在无法直接输入的场景下，用户的转写结果也不会丢失。

### 4.3 前端组件设计

前端采用 React + TypeScript 技术栈，主要负责设置界面、状态显示和用户交互。

设置窗口是用户配置应用的主要界面，包括以下配置项：ElevenLabs API Key 输入框（用于存储和验证用户的 API 密钥）、音频输入设备选择下拉框（从后端获取可用麦克风列表）、行为模式选择（直接输入文本或仅复制到剪贴板）、热键配置（显示当前配置的热键，可选是否允许自定义）。

浮动指示器是一个始终置顶、透明的迷你窗口，在录音状态下显示。这个窗口只在用户开始说话时出现，提供视觉反馈让用户知道应用正在工作。窗口设计简洁，通常显示“Listening..."文字或简单的波形动画，使用半透明背景确保不遮挡下方内容。

系统托盘图标是应用常驻后台的主要入口。托盘图标在空闲状态显示为麦克风图标，在录音状态显示为红色麦克风或带波形的图标。右键菜单包含：状态显示（就绪/录音中/处理中）、打开设置、退出应用等选项。

前端与后端的通信通过 Tauri 的命令（Commands）和事件（Events）机制实现。前端调用后端命令来启动/停止录音、获取设备列表、保存设置等。后端通过事件向前端推送状态变化、识别结果等。

### 4.4 关键模块交互流程

整个应用的交互流程涉及多个模块的协作，以下是主要的使用场景。

热键触发场景：用户按下 `Ctrl+Shift+\` 快捷键 → 全局快捷键插件捕获事件 → 发送事件到 Rust 后端 → 后端判断当前状态（空闲/录音中）→ 如果空闲，开始录音；如果正在录音，停止录音并处理结果。

录音转写场景：后端启动音频捕获 → 实时将音频数据通过 WebSocket 发送到 ElevenLabs API → 接收识别结果（可能是部分结果或最终结果）→ 如果是最终结果，触发文本插入逻辑。

文本插入场景：后端接收最终识别结果 → 尝试使用 enigo 模拟键盘输入文本 → 如果成功，任务完成；如果失败（可能当前焦点不在文本输入框），则将文本写入剪贴板 → 发送系统通知告知用户。

## 五、关键实现细节

### 5.1 全局热键注册与处理

全局热键是语音输入工具的触发入口，需要在应用启动时注册，并在整个应用生命周期中保持监听。使用 tauri-plugin-global-shortcut 插件可以轻松实现这一功能。

在 Rust 后端中注册热键的代码如下：

```rust
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

fn setup_global_shortcut(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = "ctrl+shift+\\".parse::<Shortcut>()?;

    app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            // 发送事件到应用内部处理
            app.emit("toggle-recording", ())?;
        }
    })?;

    Ok(())
}
```

热键处理逻辑需要考虑状态管理。当热键被按下时，应用需要判断当前是空闲状态还是录音状态。如果是空闲状态，则启动录音；如果是录音状态，则停止录音并处理结果。这种 toggle（切换）模式符合常见用户心理。

需要注意的是，不同操作系统对特殊键的处理可能略有差异。在 Windows 上 `Ctrl+Shift+\` 通常工作正常，但在 macOS 上可能需要调整为 `Command+Shift+\`。应用应该能够跨平台兼容这些差异。

### 5.2 音频捕获与流传输

音频捕获是实现实时语音转写的关键技术环节。使用 cpal 库可以跨平台捕获麦克风音频数据。

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use tokio::sync::mpsc;

struct AudioCapture {
    sender: mpsc::Sender<Vec<u8>>,
}

impl AudioCapture {
    fn new(sender: mpsc::Sender<Vec<u8>>) -> Self {
        Self { sender }
    }

    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device")?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        // 确保采样率符合 API 要求
        let target_sample_rate = 16000;

        let sender = self.sender.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // 将音频数据转换为16位 PCM
                        let pcm_data: Vec<u8> = data.iter()
                            .map(|&sample| {
                                let s = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                s.to_le_bytes()
                            })
                            .flatten()
                            .collect();

                        let _ = sender.blocking_send(pcm_data);
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None
                )?
            }
            _ => return Err("Unsupported sample format".into())
        };

        stream.play()?;

        // 保持 stream 存活
        std::mem::forget(stream);

        Ok(())
    }
}
```

音频数据通过通道发送到 WebSocket 处理逻辑。在发送前，需要考虑是否进行重新采样（如果设备采样率与 API 要求不同）和降噪处理。cpal 本身不提供音频处理功能，可以使用其他库（如 rubato）进行重采样。

### 5.3 WebSocket 通信实现

WebSocket 通信需要处理连接管理、消息发送和接收等多个方面。使用 tokio-tungstenite 可以实现异步 WebSocket 通信。

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

async fn start_websocket_connection(
    api_key: String,
    audio_receiver: mpsc::Receiver<Vec<u8>>,
    result_sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://api.elevenlabs.io/v1/speech-to-text/stream";

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // 发送认证消息
    let auth_msg = serde_json::json!({
        "api_key": api_key,
        "language": "auto",
        "format": "pcm_s16le"
    });
    write.send(Message::Text(auth_msg.to_string())).await?;

    // 音频数据发送循环
    let mut audio_receiver = audio_receiver;
    loop {
        tokio::select! {
            Some(audio_data) = audio_receiver.recv() => {
                // Base64 编码音频数据
                let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &audio_data);
                let msg = serde_json::json!({
                    "audio_data": encoded
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(text) = response.get("text").and_then(|t| t.as_str()) {
                                let is_final = response.get("is_final").and_then(|f| f.as_bool()).unwrap_or(false);
                                if is_final {
                                    let _ = result_sender.send(text.to_string()).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => continue,
                }
            }
        }
    }

    Ok(())
}
```

这段代码展示了 WebSocket 通信的基本框架。实际实现中还需要处理重连逻辑、错误处理、连接超时等情况。

### 5.4 文本插入与剪贴板后备

文本插入是整个应用链路的最后一环，需要精确控制。使用 enigo 库可以模拟键盘输入。

```rust
use enigo::{Enigo, Keyboard, Settings, Direction, Key};

fn type_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}

// 主处理逻辑
fn handle_transcription_result(text: String) -> Result<(), Box<dyn std::error::Error>> {
    // 首先尝试直接输入
    match type_text(&text) {
        Ok(_) => {
            println!("Text typed successfully");
            Ok(())
        }
        Err(e) => {
            println!("Failed to type text: {}, falling back to clipboard", e);
            // 备用方案：复制到剪贴板
            copy_to_clipboard(&text)?;
            // 发送系统通知
            send_notification("Transcription copied to clipboard");
            Ok(())
        }
    }
}
```

在实际实现中，可能需要添加一些延迟来确保目标应用已经准备好接收输入。此外，错误处理应该更加细致，区分不同类型的失败原因。

### 5.5 系统托盘实现

系统托盘是应用常驻后台的用户入口。使用 Tauri 2 的 tray-icon 功能可以实现这一特性。

```rust
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建托盘菜单项
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&settings_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // 加载托盘图标
    let icon = Image::from_path("icons/tray-icon.png")
        .unwrap_or_else(|_| Image::from_bytes(include_bytes!("../icons/default.png")).unwrap());

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("GhostScribe - Voice Input")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "settings" => {
                    // 打开设置窗口
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
```

托盘图标可以根据应用状态动态变化。例如，在空闲状态显示白色麦克风图标，在录音状态显示红色麦克风图标。这种视觉反馈帮助用户了解当前应用状态。

## 六、开发路线图与实现阶段

### 6.1 第一阶段：基础框架搭建

第一阶段的目标是建立 Tauri 2 项目的基础框架，实现基本的系统集成功能。

首先，需要初始化 Tauri 2 项目。可以使用官方提供的脚手架工具创建项目，选择 React 或 Vue 作为前端框架。项目创建后，配置 Cargo.toml 添加必要的依赖项，包括 tauri 核心库、全局快捷键插件、系统托盘插件等。

其次，实现系统托盘功能。创建一个基本的托盘图标和右键菜单，菜单包含“设置”和“退出”两个选项。托盘图标需要准备多个状态版本（空闲、录音中）。确保应用启动时自动隐藏主窗口，只在系统托盘区域显示图标。

第三，注册全局热键。使用 tauri-plugin-global-shortcut 注册 `Ctrl+Shift+\` 快捷键。添加日志输出验证热键是否正确触发。这一阶段不需要实际录音，只需要验证热键事件能够被正确捕获和分发。

第四，实现前端基础界面。创建一个设置窗口，包含 API Key 输入框、保存按钮等基本元素。使用 Tauri 的命令机制将设置保存到本地配置文件或系统安全存储中。

### 6.2 第二阶段：音频采集与播放

第二阶段的核心是实现麦克风音频捕获和基本处理。

首先，集成 cpal 库进行音频设备枚举。列出系统中所有可用的音频输入设备，让用户可以在设置中选择使用哪个麦克风。添加设备测试功能，让用户能够验证选定的设备是否正常工作。

其次，实现音频流捕获。以16kHz 采样率、单声道格式从选定的麦克风设备捕获音频数据。将捕获的音频数据保存到临时缓冲区，用于调试和测试。

第三，添加基本的录音状态管理。前端显示当前的录音状态（空闲、录音中）。实现开始/停止录音的逻辑控制。添加录音时长的显示。

第四，实现基本的音频波形可视化。使用 Web Audio API 在前端绘制简单的波形图，提供视觉反馈让用户知道麦克风正在工作。

### 6.3 第三阶段：API 集成

第三阶段是将 ElevenLabs Scribe v2 API 集成到应用中，实现真正的实时语音转写。

首先，实现 WebSocket 连接管理。建立与 ElevenLabs API 的 WebSocket 连接，处理连接建立、认证、心跳、断开等生命周期事件。添加重连逻辑，处理网络不稳定的情况。

其次，实现音频流传输。将捕获的音频数据编码后发送到 WebSocket 连接。优化传输节奏，避免数据积压或丢失。

第三，实现识别结果处理。解析服务器返回的 JSON 响应，提取识别文本。区分部分结果和最终结果，只在收到最终结果时触发文本插入。

第四，添加错误处理和用户反馈。处理 API 认证失败、连接超时、服务不可用等情况。向用户提供清晰的错误信息和建议。

### 6.4 第四阶段：文本插入与剪贴板后备

第四阶段实现核心的文本插入功能和备用方案。

首先，集成 enigo 库进行键盘模拟。测试在不同应用程序中的文本插入效果。处理特殊字符和换行符的输入。

其次，实现智能错误检测。检测文本插入失败的可能原因（如当前焦点不在文本输入框）。添加日志记录，帮助诊断问题。

第三，实现剪贴板后备方案。当键盘模拟失败时，自动将识别结果复制到系统剪贴板。发送系统通知告知用户结果已复制。

第四，优化用户体验。添加文本插入后的确认动画。考虑添加撤销功能，允许用户快速删除刚插入的文本。

### 6.5 第五阶段：完善与优化

第五阶段是功能完善和性能优化，确保应用达到生产就绪状态。

首先，完善设置界面。添加更多配置选项，如语言选择、音频设备偏好等。实现设置的持久化存储。

其次，优化性能。减少音频处理和转写的延迟。优化内存使用，防止长时间运行时的内存泄漏。

第三，添加多语言支持。根据用户系统语言自动选择界面语言。确保应用能够正确处理多语言识别结果。

第四，测试与调试。在不同操作系统上进行充分测试。处理各种边界情况和异常场景。

第五，准备发布。生成可执行文件和安装包。编写用户文档和帮助信息。

## 七、技术风险与应对策略

### 7.1 ElevenLabs API 可用性风险

ElevenLabs 作为商业服务，其 API 的可用性、定价和政策都可能发生变化。应对这一风险的策略包括：

首先，设计应用架构时将 API 调用抽象为独立模块，便于未来替换为其他语音识别服务。可以考虑同时支持多个语音识别后端（如 Deepgram、Whisper API 等），让用户可以选择。

其次，实现本地缓存和离线模式。虽然实时语音转写需要网络连接，但可以缓存成功的转写结果，防止因网络中断导致数据丢失。

第三，关注 API 的使用条款和定价变化。ElevenLabs 的 Scribe v2 Realtime 可能采用按用量计费的模式，需要在应用中提供用量统计和费用提醒功能。

### 7.2 跨平台兼容性风险

不同操作系统（Windows、macOS、Linux）在音频处理、键盘模拟等方面存在差异。应对策略包括：

首先，在开发过程中使用持续集成（CI）工具在多个平台上运行测试。Tauri 支持的操作系统都应该被测试覆盖。

其次，针对各平台的特定问题实现工作-around。例如，某些 Linux 发行版可能需要额外的权限配置才能访问麦克风；macOS 上的键盘模拟可能需要辅助功能的系统级授权。

第三，提供详细的平台特定安装和使用指南。帮助用户在各种环境中正确配置和应用。

### 7.3 用户体验风险

语音输入工具的用户体验至关重要，任何延迟或错误都会显著影响使用意愿。应对策略包括：

首先，将延迟优化作为核心指标。监控端到端的处理时间，持续优化关键路径。

其次，提供充分的视觉和听觉反馈。让用户始终了解应用的状态，避免不确定感。

第三，实现智能的错误恢复。当出现问题时，提供明确的错误信息和解决建议，而非无声失败。

第四，收集用户反馈并持续迭代。提供反馈渠道，倾听用户意见并据此改进产品。

## 八、总结与展望

本技术方案详细阐述了基于 ElevenLabs Scribe v2 Realtime API 和 Tauri 2 框架构建 Wispr Flow 类语音输入工具的完整路径。通过深入分析 ElevenLabs API 的技术规格和通信协议，借鉴 Wispr Flow 等竞品的成功经验，设计了包括音频捕获、WebSocket 通信、文本插入、系统托盘、全局热键等核心功能模块的技术架构。

Tauri 2 框架为本项目提供了理想的开发基础，其轻量级、高性能、跨平台的特性非常适合这类常驻后台的系统工具。Rust 后端负责高性能的音频处理和系统交互，TypeScript 前端提供现代化的用户界面，两者通过 Tauri 的命令和事件机制紧密协作。

展望未来，这类语音输入工具还有广阔的发展空间。随着语音识别技术的持续进步，识别准确率和响应速度将进一步提升。大型语言模型的集成将使转写结果更加智能，不仅能准确识别语音，还能自动完成标点、格式化、甚至语义修正。多模态输入的融合也将带来更多创新可能，如语音与手势、眼神的结合，为用户提供更加自然高效的输入方式。

本方案为实现这一创新工具奠定了坚实的技术基础，期待后续能够成功落地并持续迭代，为广大用户带来卓越的语音输入体验。
