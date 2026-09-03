# Smart Error Notebook 开发者手册

> 欢迎参与开发！本文档帮助你快速上手项目。

---

## 📑 目录

1. [环境要求](#-环境要求)
2. [本地开发环境搭建](#-本地开发环境搭建)
3. [项目结构](#-项目结构)
4. [前端开发](#-前端开发)
5. [Rust 后端开发](#-rust-后端开发)
6. [数据库开发](#-数据库开发)
7. [历史同步服务器](#-历史同步服务器)
8. [构建与发布](#-构建与发布)
9. [代码规范](#-代码规范)
10. [贡献指南](#-贡献指南)

---

## 🔧 环境要求

| 工具 | 最低版本 | 安装方式 |
|------|----------|----------|
| Rust | 1.77+ | [rustup](https://rustup.rs/) |
| Node.js | 22.13+ | [nvm](https://github.com/nvm-sh/nvm) 或官方安装包 |
| pnpm | 11+ | `npm install -g pnpm@^11.0.0` |
| Tauri CLI | 2.x | 由 pnpm 管理 |
| Android SDK | 34+ | Android Studio 或命令行 |
| Android NDK | 26+ | 通过 SDK Manager 安装 |
| Gradle | 8.14+ | 由 Tauri 自动管理 |

> **Android 开发须知**：构建 Android 端需要安装 Android Studio（或至少 Android SDK + NDK）。
> 安装后设置环境变量：
> ```bash
> export ANDROID_HOME=$HOME/Android/Sdk
> export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/26.3.11579264
> export PATH=$PATH:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin
> ```

### 系统依赖

**Ubuntu/Debian（桌面端 + Android）：**

```bash
# 桌面端依赖
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Android 额外依赖（用于交叉编译 Rust → Android）
sudo apt install pkg-config libc6-dev-amd64-cross
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

**Fedora:**

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libxdo-devel libappindicator-gtk3-devel librsvg2-devel
```

**Arch Linux:**

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg
```

**macOS:** 不需要额外系统依赖（Xcode CLI tools 即可）

**Windows:** 安装 [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

---

## 🚀 本地开发环境搭建

```bash
# 1. 克隆仓库
git clone https://github.com/zpb911km/SmartErrorNotebook.git
cd SmartErrorNotebook

# 2. 安装前端依赖
pnpm install

# 3. 启动开发模式
pnpm tauri dev

# 或者只启动前端（不带 Tauri 壳）
pnpm dev
```

### 开发模式说明

`pnpm tauri dev` 会：
1. 启动 Vite 开发服务器（端口 1420）
2. 编译 Rust 后端
3. 打开 Tauri 桌面窗口，加载前端页面
4. 自动打开 DevTools（调试模式）
5. 前端代码热更新，Rust 代码需重启

> **注意**：首次运行时会自动创建 SQLite 数据库并执行所有迁移。

---

## 📁 项目结构

```
SmartErrorNotebook/
├── index.html                  # 入口 HTML
├── package.json                # 前端依赖与脚本
├── vite.config.ts              # Vite 构建配置
├── tsconfig.json               # TypeScript 配置
├── .prettierrc                 # 代码格式化配置
├── .prettierignore             # 格式化忽略文件
├── .gitignore                  # Git 忽略规则
├── sync.md                     # 同步协议文档
├── LICENSE                     # MIT 许可证
│
├── src/                        # ── 前端 Vue 3 应用
│   ├── main.ts                 # 应用入口
│   ├── App.vue                 # 根组件（主题、布局）
│   ├── vite-env.d.ts           # Vite 类型声明
│   │
│   ├── router/
│   │   └── index.ts            # Vue Router 路由定义（12 个页面）
│   │
│   ├── views/                  # 页面组件
│   │   ├── Home.vue            # 首页（轮播、统计概览）
│   │   ├── Add.vue             # 添加错题（拍照/选图/AI 识别）
│   │   ├── Manage.vue          # 错题管理（搜索、筛选、列表）
│   │   ├── Manage-Detail.vue   # 错题详情（编辑、附件、标签）
│   │   ├── Preview.vue         # 复习计划（筛选待复习卡片）
│   │   ├── Review-Detail.vue   # 复习执行（展示题目、评分）
│   │   ├── Profile.vue         # 个人主页（统计、SRS 图表）
│   │   ├── Settings.vue        # 设置（主题、AI、导出配置）
│   │   ├── Sync.vue            # 远程服务停用说明
│   │   └── MarkdownTextareaTest.vue  # 组件测试页
│   │
│   ├── components/             # 复用组件
│   │   ├── TopBar.vue          # 顶部导航栏
│   │   ├── BottomNav.vue       # 底部导航
│   │   ├── Card.vue            # 错题卡片
│   │   ├── CameraModal.vue     # 相机拍照模态框
│   │   ├── ImageEditor.vue     # 图片裁剪编辑
│   │   ├── MarkdownTextarea.vue # Markdown 编辑器（预览 + 编辑）
│   │   ├── ErrorTagSelector.vue # 错因标签选择器
│   │   ├── SubjectSelector.vue # 科目选择器
│   │   ├── SourceSelector.vue  # 来源选择器
│   │   ├── FilterNav.vue       # 筛选导航
│   │   ├── ExportModal.vue     # 导出模态框
│   │   ├── ExportPreview.vue   # 导出预览
│   │   ├── ImportModal.vue     # 导入模态框
│   │   ├── SyncOverlay.vue     # 同步遮罩层
│   │   ├── Notification.vue    # 通知消息
│   │   ├── PromptEditor.vue    # 提示词编辑器
│   │   ├── Button.vue          # 按钮组件
│   │   ├── Tag.vue             # 标签组件
│   │   └── Icon.vue            # Lucide 图标封装
│   │
│   ├── api/legacy/             # 对 Rust 后端的 legacy API 调用
│   │   ├── index.ts            # 统一导出
│   │   ├── errorQuestions.ts   # 错题 CRUD
│   │   ├── errorTags.ts        # 错因标签
│   │   ├── subjects.ts         # 科目
│   │   ├── sources.ts          # 来源
│   │   ├── attachments.ts      # 附件
│   │   ├── srs.ts              # SRS 复习
│   │   ├── srsData.ts          # SRS 数据
│   │   └── sync.ts             # 本地同步元数据维护
│   │
│   ├── services/
│   │   ├── index.ts            # 统一导出
│   │   ├── llm.ts              # LLM 服务（OpenAI 格式，单例）
│   │   ├── reviewStore.ts      # 复习状态管理
│   │   └── shareStore.ts       # 分享状态管理
│   │
│   ├── utils/
│   │   ├── exportFile.ts       # 文件导出通用工具
│   │   ├── exportHtml.ts       # HTML 导出（含 KaTeX 公式）
│   │   ├── exportPdf.ts        # PDF 导出（html2pdf.js）
│   │   ├── exportJson.ts       # JSON 导出
│   │   ├── importJson.ts       # JSON 导入（去重）
│   │   ├── formatter.ts        # 格式化工具
│   │   ├── imageCompression.ts # 图片压缩
│   │   ├── inquiry.ts          # AI 查询逻辑
│   │   ├── notification.ts     # 通知工具
│   │   └── shareContent.ts     # 分享内容工具
│   │
│   ├── types/
│   │   ├── index.ts            # 空占位文件
│   │   └── legacy.ts           # Legacy TypeScript 类型定义
│   │
│   ├── directives/
│   │   ├── ripple.ts           # 水波纹点击效果
│   │   └── scrollReveal.ts     # 滚动显示动画
│   │
│   ├── composables/
│   │   └── useCountUp.ts       # 数字递增动画
│   │
│   └── styles/
│       ├── global.css          # 全局样式
│       └── theme.css           # 主题变量（明暗）
│
├── src-tauri/                  # ── Rust 后端（Tauri 2）
│   ├── Cargo.toml              # Rust 依赖
│   ├── build.rs                # 构建脚本
│   ├── tauri.conf.json         # Tauri 配置
│   ├── capabilities/default.json # 权限配置
│   ├── crates/migration/       # 独立 SeaORM 迁移 crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # 迁移注册
│   │       ├── main.rs         # 迁移 CLI
│   │       └── m*.rs           # 历史及规范化迁移
│   └── src/
│       ├── main.rs             # 入口
│       ├── lib.rs              # Tauri Builder 与 AppState
│       ├── command/            # 命令注册和 legacy IPC 兼容层
│       │   ├── mod.rs
│       │   └── legacy/         # CRUD、SRS、同步和文件命令
│       ├── data/               # SeaORM 数据访问实现
│       │   ├── database/
│       │   │   ├── connection.rs
│       │   │   └── entity/     # 规范化实体及交叉引用实体
│       │   └── repository/     # SeaORM 仓储与事务执行器
│       ├── domain/             # 领域层
│       │   ├── model/          # 内部领域模型
│       │   └── repository/     # 仓储 traits 和 legacy 契约模型
│       └── srs/                # SDR 算法引擎
│           └── mod.rs          # 核心算法实现
│
├── server/                     # ── 历史同步服务器（当前不支持）
│   ├── app.py                  # Flask 应用（API + 管理页面）
│   ├── requirements.txt        # Python 依赖
│   └── templates/
│       └── admin.html          # 管理后台页面
│
├── src-tauri/gen/android/      # ── Android 原生项目（Gradle）
│   ├── app/build.gradle.kts    # Android 应用构建配置
│   ├── build.gradle.kts        # 顶层 Gradle 配置
│   ├── settings.gradle         # 项目设置
│   ├── gradlew                 # Gradle Wrapper
│   └── app/src/                # Android 原生代码
│       ├── main/AndroidManifest.xml   # 清单文件（权限声明）
│       └── main/java/          # Kotlin/Java 桥接代码
│
└── docs/                       # ── 文档
    ├── USER_GUIDE.md           # 用户手册
    ├── DEVELOPMENT.md          # 本文件
    ├── ARCHITECTURE.md         # 架构设计
    ├── SRS_ALGORITHM.md        # 复习算法
    ├── API_REFERENCE.md        # API 参考
    └── SYNC_PROTOCOL.md        # 同步协议
```

---

## 🖥️ 前端开发

### 技术选型

- **框架**: Vue 3 (Composition API + `<script setup>`)
- **语言**: TypeScript（`strict` 模式）
- **构建**: Vite 6
- **路由**: Vue Router 4 (Hash 模式)
- **动效**: GSAP 3
- **公式渲染**: KaTeX
- **代码高亮**: Highlight.js

### 路由表

| 路径 | 页面 | 标题 |
|------|------|------|
| `/` | 重定向到 `/home` | — |
| `/home` | Home | 首页 |
| `/add` | Add | 添加错题 |
| `/manage` | Manage | 错题管理 |
| `/manage-detail/:id` | ManageDetail | 错题详情管理 |
| `/review` | Preview | 复习计划 |
| `/review-detail` | ReviewDetail | 复习详情 |
| `/stats` | Profile | 个人主页 |
| `/settings` | Settings | 设置 |
| `/sync` | Sync | 远程服务停用说明 |
| `/community` | redirect | 重定向到同步停用说明 |
| `/markdown-test` | MarkdownTextareaTest | Markdown 组件测试 |

### 调用 Rust 后端

前端通过 `@tauri-apps/api` 的 `invoke` 调用 Rust 命令：

```typescript
import { invoke } from '@tauri-apps/api/core'

// 示例：获取所有科目
const subjects = await invoke('legacy_get_subjects')

// 示例：创建错题
const newQuestion = await invoke('legacy_create_question', {
  prompt: '题目内容',
  type_: '多选题',
  subject_id: 'xxx'
})
```

> Rust 命令的完整清单见 [API_REFERENCE.md](API_REFERENCE.md)

### LLM 服务

`src/services/llm.ts` 提供单例模式的 LLM 调用，兼容 OpenAI API 格式：

```typescript
import { LLMService } from '../services/llm'

const llm = LLMService.getInstance()
const response = await llm.call([
  { role: 'user', content: 'Hello' }
])
```

---

## 🦀 Rust 后端开发

### 技术选型

- **框架**: Tauri 2
- **ORM**: SeaORM (SQLite)
- **序列化**: Serde + Serde JSON
- **异步**: Tokio (full features)

### 添加新命令

1. 在 `src-tauri/src/command/legacy/` 下创建模块（或添加到已有模块）
2. 在 `request/`、`response/` 中定义兼容 IPC 结构，并实现标注 `#[tauri::command]` 的处理器
3. 从 `command/legacy/mod.rs` 导出处理器
4. 在 `src-tauri/src/command/mod.rs` 的 `register_command()` 中注册

```rust
// 1. 实现命令
#[tauri::command]
pub async fn my_new_command(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.repository_transaction_executor
        .execute(|factory, _| Box::pin(async move {
            // 通过 factory 获取仓储并完成同一事务内的操作
            Ok("done".to_string())
        }))
        .await
}

// 2. 注册命令（command/mod.rs）
self.invoke_handler(tauri::generate_handler![
    // ... 已有命令
    legacy::my_new_command,
])
```

### 数据库连接

数据库连接通过 `AppState` 管理，所有命令通过 `State<AppState>` 获取：

```rust
pub struct AppState {
    pub repository_transaction_executor: SeaOrmRepositoryTransactionExecutor,
}
```

数据库文件位置：
- **桌面和移动端**：系统 AppData 下的 `SmartErrorNotebook/data/database.db`
- **测试**：测试夹具使用单连接内存 SQLite

---

## 🗄️ 数据库开发

### 实体定义

使用 SeaORM 的 `DeriveEntityModel` 派生宏：

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "subject")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
    pub sync_status: String,
    pub sync_version: i64,
    pub name: String,
    pub color: String,
}
```

题目通过 `source_id` 间接归属科目；标签和附件分别通过 `question_tag_cross_ref`、`question_attachment_cross_ref` 表示多对多关系。

### 迁移管理

迁移文件位于 `src-tauri/crates/migration/src/`，命名规则：

```
mYYYYMMDD_NNNNNN_description.rs
```

示例：`m20260812_100149_normalize_database.rs`

**新建迁移步骤：**

1. 在 migration crate 中创建迁移模块
2. 实现 `up()`、`down()`，并按数据安全需求决定是否启用事务
3. 在 `crates/migration/src/lib.rs` 中声明模块并按顺序注册
4. 为数据变换、失败回滚和兼容升级路径增加迁移测试

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 建表/改表逻辑
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚逻辑
    }
}
```

> 注意：数据库在应用启动时自动执行迁移，无需手动运行命令。

开发时也可在 `src-tauri/` 下通过 `cargo run -p migration -- -u <database-url> status|up` 检查或执行迁移。

---

## 🌐 历史同步服务器

`server/` 仅保留为历史代码，与当前规范化数据库模型和多对多关系不兼容。
客户端不会连接它，开发和发布流程也不支持部署它。未来同步实现应先遵守
[同步协议草案](SYNC_PROTOCOL.md) 的复合记录标识、关系集合和版本规则。

---

## 📦 构建与发布

### 桌面端打包

```bash
# 构建当前桌面平台的安装包（Windows .msi / macOS .dmg / Linux .deb）
pnpm tauri build
```

构建产物在 `src-tauri/target/release/bundle/` 目录下。

### Android 构建

#### 环境要求

确保已配置以下环境变量：

```bash
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/26.3.11579264
export PATH=$PATH:$ANDROID_HOME/platform-tools

# 添加 Rust Android 编译目标
rustup target add aarch64-linux-android   # 主流 Android 设备
rustup target add armv7-linux-androideabi # 老旧设备
rustup target add x86_64-linux-android    # 模拟器
```

#### 初始化（仅首次）

```bash
pnpm tauri android init
```

这将生成 `src-tauri/gen/android/` 目录，包含完整的 Gradle 项目。

#### 运行到设备

```bash
# 连接 Android 设备（USB 调试），或打开 Android 模拟器
pnpm tauri android dev
```

#### 构建 APK / AAB

```bash
# Debug APK（可直接安装测试）
pnpm tauri android build --debug

# Release APK（需要签名配置）
pnpm tauri android build
```

#### 构建产物位置

```
src-tauri/gen/android/app/build/outputs/
├── apk/     # APK 文件（直接安装）
└── bundle/  # AAB 文件（Google Play 上传）
```


### 发布前检查清单

- [ ] 前端构建无报错（`pnpm build`）
- [ ] Rust 编译无警告
- [ ] 版本号更新（`package.json` + `src-tauri/Cargo.toml` + `src-tauri/tauri.conf.json`）
- [ ] 更新文档中的版本信息
- [ ] 测试数据库迁移在新环境可正常执行
- [ ] Android：签名密钥已配置
- [ ] Android：在真机上测试过相机和文件分享功能

---

### Android 权限说明

Android 端需要以下权限，在 `src-tauri/gen/android/app/src/main/AndroidManifest.xml` 中声明：

| 权限 | 用途 | 类别 |
|------|------|:----:|
| `CAMERA` | 拍照录入错题 | 危险权限 |
| `INTERNET` | LLM API 调用 | 普通 |
| `ACCESS_NETWORK_STATE` | 检查网络状态 | 普通 |
| `WRITE_EXTERNAL_STORAGE` | 导出文件保存（Android 10 以下） | 危险权限 |
| `POST_NOTIFICATIONS` | 复习提醒（计划中） | 危险权限 |

权限在运行时动态申请（使用 Tauri Dialog Plugin），首次使用相关功能时弹出授权请求。

---

## 📐 代码规范

### TypeScript / JavaScript

- 使用 TypeScript 严格模式
- 使用 Prettier 格式化（配置见 `.prettierrc`）

```bash
# 格式化所有前端代码
pnpm format
```

### Rust

- 遵循 Rust 2021 edition 风格
- 使用 `cargo fmt` 和 `cargo clippy`

### Git 提交

推荐使用以下提交类型：

| 类型 | 用途 |
|------|------|
| `feat:` | 新功能 |
| `fix:` | 修复 |
| `docs:` | 文档 |
| `refactor:` | 重构 |
| `perf:` | 性能优化 |
| `chore:` | 杂项（构建、依赖等） |

示例：`feat: 添加按来源筛选错题功能`

---

## 🤝 贡献指南

### 流程

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交改动 (`git commit -m 'feat: 添加某个功能'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 分支命名

- `feature/xxx` — 新功能
- `fix/xxx` — 修复
- `docs/xxx` — 文档改进
- `refactor/xxx` — 重构

### 报告 Issue

提交 Issue 时请包含：
- 问题描述
- 复现步骤
- 预期行为与实际行为
- 环境信息（OS、版本号）
- 截图（如适用）

---

> 有任何问题？请提交 [GitHub Issue](https://github.com/zpb911km/SmartErrorNotebook/issues)
