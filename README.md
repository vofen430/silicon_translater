# Silicon Translator（Windows 桌面划词翻译器）


本仓库提供一个基于 **Tauri（TypeScript/JavaScript 前端 + Rust 后端）** 的 Windows 桌面 AI 划词翻译器 MVP 骨架，满足：

- 常驻托盘 + 悬浮按钮 + 悬浮翻译窗
- Windows 系统级划词检测（UI Automation + 剪贴板兜底）
- SiliconFlow OpenAI 兼容接口翻译
- 可扩展至 OCR 截图翻译、UIAccess、管理员模式
- **必须支持 Windows x86（32 位）构建与发布**

> 说明：当前为 MVP 骨架版本，已搭好结构、通信与关键模块，UIA/钩子/剪贴板等系统能力留有实现位置。

---

## 项目结构（强制分层）

```
.
├── src/                     # 前端 UI
│   ├── index.html
│   ├── main.ts
│   └── styles.css
├── src-tauri/               # Rust 后端 + Tauri
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── api_client/      # SiliconFlow API 调用
│       ├── core/            # 业务编排
│       ├── platform_windows/# Win32/UIA/Hook
│       ├── storage/         # 配置与凭据
│       └── ui_bridge/       # 前后端通信
├── package.json
├── tsconfig.json
└── vite.config.ts
```

### 关键模块职责

- `core/`：业务编排、状态管理、事件分发（选择变化 → UI）
- `platform_windows/`：WinEventHook/低级鼠标钩子/UIA/剪贴板兜底、DPI/多屏坐标换算
- `api_client/`：SiliconFlow OpenAI Chat Completions 调用、错误分类、重试
- `storage/`：配置管理、Windows Credential Manager / DPAPI 安全存储、剪贴板备份
- `ui_bridge/`：Tauri commands 与事件通知

---

## 核心流程时序（文字描述）

1. **系统监听**（WinEventHook + 鼠标抬起）触发 `SelectionWatcher`。
2. `SelectionWatcher` 尝试路径 A（UIA Selection），失败后尝试路径 B（剪贴板备份/恢复）。
3. 成功获得文本后触发 `selection-event` 通知前端。
4. 前端根据选择显示悬浮按钮（后续里程碑实现），点击后呼出翻译窗。
5. 前端调用 `translate` command → `ApiClient` → SiliconFlow API → 返回译文。
6. UI 更新译文，提供复制/朗读/重新翻译。

---

## Windows x86（32 位）构建说明（必须支持）

### 依赖

- **WebView2 Runtime（x86）**
  - 推荐安装：`https://developer.microsoft.com/en-us/microsoft-edge/webview2/`
  - 生产环境可使用 Evergreen 安装器或打包固定版本。

### 构建步骤

```bash
# 安装前端依赖
npm install

# 构建前端
npm run build

# 安装 Rust 32 位目标
rustup target add i686-pc-windows-msvc

# 构建 Tauri x86 版本
cargo tauri build --target i686-pc-windows-msvc
```

> 在 CI 或发行脚本中，请确保使用 **MSVC 32 位工具链** 和 **x86 WebView2**。

---

## MVP 验收脚本（手动）

1. 打开 **Notepad**，输入英文并选中文本 → 观察悬浮按钮（里程碑 2）。
2. 点击悬浮按钮 → 翻译窗弹出 → 译文可复制。
3. 在浏览器与 PDF 阅读器中重复上述流程。

---

## 里程碑规划（建议）

- **里程碑 1**：托盘 + 翻译窗 + SiliconFlow 翻译链路跑通（手动粘贴翻译）。
- **里程碑 2**：检测划词并弹出按钮（先用鼠标位置）。
- **里程碑 3**：UIA bounding rect + 剪贴板兜底完善 + DPI 多屏修正。
- **里程碑 4**：设置页 + 安全存储 + 错误处理打磨。
- **里程碑 5**：截图 OCR（预留接口）。

---

## 安全与权限策略

- API Key 存储在 Windows Credential Manager（`storage/`）。
- 不在日志输出完整 API Key。
- 支持“可选管理员运行模式”覆盖 UIPI 隔离（需用户手动以管理员运行）。
- UIAccess 作为可选增强，需签名 + 安装目录约束（详见文档扩展）。

---

## 关键文件列表

- `src-tauri/src/main.rs`：Tauri 初始化、托盘菜单、事件桥接
- `src-tauri/src/api_client/mod.rs`：SiliconFlow Chat Completions 调用
- `src-tauri/src/platform_windows/mod.rs`：UIA/剪贴板/钩子入口与定位逻辑
- `src-tauri/src/storage/mod.rs`：配置、Credential Manager、剪贴板备份
- `src-tauri/src/ui_bridge/mod.rs`：Tauri commands 与前端事件
- `src/main.ts`：UI 交互、翻译调用、事件监听

---

## 运行（开发模式）

```bash
npm install
npm run dev
cargo tauri dev
```

> Windows 上请确保 WebView2 Runtime 已安装。
