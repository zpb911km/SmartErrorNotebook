
<p align="center">
  <img src="app-icon.png" alt="Smart Error Notebook" width="120" />
</p>

<h1 align="center">Smart Error Notebook</h1>
<p align="center">
  <b>智能错题本</b> —— 让每一次错误，都成为进步的阶梯
</p>

<p align="center">
  <a href="#-功能速览">功能</a> •
  <a href="#-快速开始">快速开始</a> •
  <a href="#-文档导航">文档</a> •
  <a href="#-技术栈">技术栈</a> •
  <a href="#-许可证">许可证</a>
</p>

---

## ✨ 功能速览

| 功能 | 说明 |
|------|------|
| 📸 **拍照录入** | 拍照或选择图片，自动识别错题内容 |
| 🤖 **AI 智能分析** | 对接 OpenAI 兼容 API，自动提取题干、答案、解析 |
| 📂 **分类管理** | 按科目、来源（书/章节/知识点）、错因标签归类 |
| 🧠 **间隔重复复习** | SDR 算法驱动，自适应调整复习频率 |
| 🔄 **多设备同步** | 自建同步服务器，离线优先，冲突可解 |
| 📤 **多格式导出** | HTML / JSON 导出；移动端支持 Share（微信/QQ 等） |
| 🌐 **错题社区** | 分享错题，浏览他人分享 |
| 🌗 **明暗主题** | 浅色 / 深色 / 跟随系统 |
| 📱 **跨平台** | 桌面端（Win/Linux）+ 移动端（Android）|

---

## 🚀 快速开始

### 使用软件

请前往release页面下载合适的安装包, 安装即可本地使用

云端同步是可选功能, 需要自备服务器和 IP / 域名

### 桌面端

```bash
# 前置要求：Rust toolchain, Node.js ≥ 18, pnpm
git clone https://github.com/zpb911km/SmartErrorNotebook.git
cd SmartErrorNotebook
pnpm install
pnpm tauri dev
```

### Android 移动端

```bash
# 前置要求：Rust toolchain + Android SDK/NDK
pnpm tauri android init
pnpm tauri android dev    # USB 连接设备运行
pnpm tauri android build  # 生成 APK/AAB
```

> 详细构建说明见 [开发者手册](docs/DEVELOPMENT.md#-android-构建)

### 同步服务器（可选）

```bash
cd server
pip install -r requirements.txt
python app.py
```

> 详细安装步骤见 [用户手册](docs/USER_GUIDE.md#-安装与启动)

---

## 🗺️ 文档导航

| 如果你想... | 请阅读 |
|-------------|--------|
| 📖 **安装并使用这个应用** | [用户手册](docs/USER_GUIDE.md) |
| 🛠️ **参与开发或贡献代码** | [开发者手册](docs/DEVELOPMENT.md) |
| 🏗️ **了解整体架构设计** | [架构设计](docs/ARCHITECTURE.md) |
| 🔗 **部署同步服务器** | [同步协议](docs/SYNC_PROTOCOL.md) |
| 🧠 **理解复习算法原理** | [SRS 算法](docs/SRS_ALGORITHM.md) |
| 📋 **查看 API 接口清单** | [API 参考](docs/API_REFERENCE.md) |

---

## 🧱 技术栈

| 层 | 技术 |
|----|------|
| **前端框架** | Vue 3 + TypeScript + Vite 6 |
| **UI 组件** | Lucide Icons, GSAP 动效, KaTeX 公式渲染, Highlight.js 代码高亮 |
| **桌面壳** | Tauri 2 (Rust) |
| **移动端** | Android（Gradle） |
| **本地数据库** | SQLite via SeaORM |
| **同步服务器** | Flask + SQLAlchemy（可选部署） |
| **移动端分享** | Web Share API（分享到微信/QQ 等） + Tauri Share Plugin |
| **包管理** | pnpm |

---

## 📄 许可证

[MIT](LICENSE) © 2026 Smart Error Notebook contributors

---

<p align="center">
  <sub>由 <code>zpb911km</code>、<code>fly-960452909</code>、<code>dimixun</code>、<code>BakeReisen</code> 构建</sub>
</p>
