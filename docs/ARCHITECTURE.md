# Smart Error Notebook 架构设计

> 本文档描述项目的整体架构、分层设计、关键数据流与设计决策。

---

## 📑 目录

1. [总体架构](#-总体架构)
2. [数据流详解](#-数据流详解)
3. [目录详解](#-目录详解)
4. [关键设计决策](#-关键设计决策)
5. [数据结构关系](#-数据结构关系)
6. [同步架构](#-同步架构)
7. [技术债与 FIXME](#-技术债与-fixme)

---

## 🏛️ 总体架构

本项目采用 **三层架构**：前端展示层 → Rust 业务逻辑层 → 本地存储层，外加可选的同步服务器层。

```mermaid
graph TB
    %% ══════════ 前端展示层 ══════════
    subgraph Frontend["🖥️ 前端展示层 (Vue 3 + TypeScript)"]
        direction TB

        subgraph V["Views（12 页面）"]
            V1[Home] --- V2[Add]
            V3[Manage] --- V4[Manage-Detail]
            V5[Preview] --- V6[Review-Detail]
            V7[Profile] --- V8[Settings]
            V9[Sync] --- V10[Community]
        end

        subgraph C["Components（18 组件）"]
            C1[TopBar / BottomNav]
            C2[CameraModal / ImageEditor]
            C3[MarkdownTextarea / ErrorTagSelector]
            C4[ConflictResolver / Card]
            C5[ExportModal / ExportPreview]
            C6[ImportModal]
        end

        subgraph S["Services / Utils"]
            S1[apis/ invoke 调用层]
            S2[types/ TS 接口定义]
            S3[services/llm.ts AI 识别]
            S4[utils/export* 导出]
            S5[utils/import* 导入]
        end

        V -->|使用| C
        V -->|调用| S
        C --> S

        Export[ExportModal.vue] -.->|桌面: Tauri save<br/>移动端: Share btn| S4
        Camera[CameraModal.vue] -.->|桌面: MediaDevices<br/>移动端: 原生相机| V2
    end

    %% ══════════ Tauri IPC ══════════
    Frontend -->|Tauri IPC invoke / events<br/>JSON via Serde| Bridge

    subgraph Bridge["Android 原生桥接层"]
        BM[AndroidManifest.xml<br/>权限声明]
        BP1[tauri-plugin-share 文件分享]
        BP2[tauri-plugin-fs 文件系统]
        BP3[tauri-plugin-dialog 对话框]
        BE["# [cfg_attr(mobile, tauri::mobile_entry_point)]"]
    end

    Bridge --> Backend

    %% ══════════ Rust 业务逻辑层 ══════════
    subgraph Backend["⚙️ Rust 业务逻辑层 (Tauri 2 + SeaORM)"]
        direction TB

        subgraph Cmd["commands/（Tauri 命令）"]
            CM1[subject.rs] --- CM2[error_question.rs]
            CM3[error_tag.rs] --- CM4[source.rs]
            CM5[attachment.rs] --- CM6[srs_data.rs]
            CM7[sync.rs] --- CM8[user_config.rs]
        end

        subgraph DAL["database/（数据访问层）"]
            D1[connection.rs 数据库连接]
            D2[entities/ ORM 实体定义]
            D3[migrations/ 14 个迁移文件]
        end

        subgraph SRS["srs/（SDR 算法引擎）"]
            R1[mod.rs]
            R2[review_card 复习核心]
            R3[predict_retrievability 预测]
            R4[compute_next_interval 间隔]
        end

        Cmd --> DAL
        SRS --> Cmd
    end

    %% ══════════ 本地存储层 ══════════
    Backend -->|SQLite 文件| Storage

    subgraph Storage["💾 本地存储层 (SQLite)"]
        T1[user_config 用户配置]
        T2[subjects 科目]
        T3[error_questions 错题]
        T4[srs_data 复习数据]
        T5[sources 来源]
        T6[error_tags 错因标签]
        T7[attachments 附件]
    end

    Storage -.-> Location
    Location[数据库位置按平台:<br/>Win: %APPDATA%/<br/>Mac: ~/Library/<br/>Linux: ~/.local/share/]

    %% ══════════ 同步服务器 ══════════
    Storage -.->|HTTP 可选| SyncServer

    subgraph SyncServer["🌐 同步服务器（可选）<br/>(Flask + SQLAlchemy)"]
        API1[POST /api/auth/validate]
        API2[POST /api/sync/handshake]
        API3[POST /api/sync/push]
        API4[POST /api/sync/pull]
        API5[GET /admin 管理后台]

        DB1[(user_auth)]
        DB2[(records)]
        DB3[(shared_questions)]
    end

    API1 --> DB1
    API2 --> DB2
    API3 --> DB2
    API4 --> DB2
    API5 --> DB1
```

---

## 🔄 数据流详解

### 典型场景：添加一道错题

```mermaid
sequenceDiagram
    participant User as 用户
    participant App as 前端界面
    participant LLM as LLM API
    participant Rust as Rust 后端
    participant DB as SQLite

    User->>App: ① 拍照/选图
    App->>LLM: ② AI 识别 (HTTP)
    LLM-->>App: ← JSON 响应（题干/答案/解析）
    User->>App: ③ 确认信息
    User->>App: ④ 点击保存
    App->>Rust: invoke('create_question')
    Rust->>DB: 生成 UUID + 写入
    DB-->>Rust: 返回新记录
    Rust-->>App: 返回结果
    App->>Rust: invoke('create_attachments_for_question')
    App->>Rust: invoke('create_srs_data')
    App->>Rust: invoke('create_error_tags_for_question')
    Rust->>DB: 批量写入
    DB-->>Rust: 写入完成
    Rust-->>App: 全部完成
    App->>User: ⑤ 页面跳转
```

### 典型场景：执行一次复习

```mermaid
flowchart TD
    A[① 进入复习页] --> B{invoke get_due_questions}
    B --> C[Rust 查询 SRS 数据]
    C --> C1[next_review_at ≤ now]
    C1 --> C2[deleted_at IS NULL]
    C2 --> D[返回待复习列表]
    D --> E[② 逐题复习<br/>显示题目 → 回忆 → 显示答案]
    E --> F[③ 滑动评分<br/>反馈值 0.0 ~ 1.0]
    F --> G[invoke submit_review_result]
    G --> H{进入 srs/mod.rs}
    H --> H1[predict_retrievability]
    H1 --> H2[更新稳定性 S]
    H2 --> H3[更新难度 D]
    H3 --> H4[计算下次复习间隔]
    H4 --> H5[更新数据库]
    H5 --> I[④ 下一题]
    I --> E
```

---

## 📂 目录详解

### 前端 (`src/`)

| 目录 | 职责 | 关键约定 |
|------|------|----------|
| `views/` | 页面组件，对应路由 | 每个 `.vue` 一个页面，命名 PascalCase |
| `components/` | 可复用 UI 组件 | 无业务逻辑，通过 props/events 通信 |
| `apis/` | `invoke()` 封装层 | 每个 Rust 实体对应一个文件 |
| `services/` | 状态管理 + 业务服务 | LLM 服务为单例模式 |
| `utils/` | 纯函数工具 | 不含副作用 |
| `types/` | TypeScript 接口定义 | 前后端契约 |
| `directives/` | Vue 自定义指令 | — |
| `styles/` | 全局样式 + 主题变量 | 主题通过 CSS 变量切换 |

### Rust 后端 (`src-tauri/src/`)

| 目录 | 职责 | 关键约定 |
|------|------|----------|
| `commands/` | Tauri 命令处理器 | 每个文件 ≈ 一个实体，函数标注 `#[tauri::command]` |
| `database/entities/` | SeaORM 实体 | `#[derive(DeriveEntityModel)]` |
| `database/migrations/` | 数据库迁移 | 命名 `mYYYYMMDD_NNNNNN_desc.rs` |
| `srs/` | 核心复习算法 | 纯函数，不依赖 Tauri/数据库 |

### 同步服务器 (`server/`)

| 文件 | 职责 |
|------|------|
| `app.py` | Flask 应用，包含所有 API 路由和数据模型 |
| `requirements.txt` | Python 依赖 |
| `templates/admin.html` | 管理后台页面 |

---

## 📱 移动端架构

### Android 构建管线

```mermaid
graph LR
    subgraph FrontendBuild["TypeScript / Vue 3"]
        A[Vite 构建] --> A1[dist/]
    end

    subgraph AndroidNative["Kotlin / Java (Android)"]
        B[Android Activity<br/>WebView wrapper]
    end

    subgraph RustBuild["Rust 后端"]
        C[Rust 编译] --> C1[SeaORM + SQLite]
    end

    A1 -->|Tauri 打包| C
    B -->|Tauri 2 框架| C

    C --> D[Android APK / AAB]

    subgraph Output["构建产物"]
        D1[• Rust → .so 动态库 aarch64/armv7]
        D2[• 嵌入 Android assets]
        D3[• WebView 加载 dist]
        D4[• Rust ↔ JS via Tauri IPC]
    end

    D --> D1
    D --> D2
    D --> D3
    D --> D4
```

### 移动端与桌面端的差异点

| 维度 | 桌面端 | Android 端 |
|------|--------|------------|
| **窗口** | 独立窗口 (800×600) | 全屏 Activity，无窗口概念 |
| **文件交互** | Tauri save/open dialog | Tauri Dialog Plugin + Web Share API |
| **分享** | 隐藏分享按钮 | `navigator.share()` 调用系统分享 |
| **相机** | `navigator.mediaDevices.getUserMedia()` | 同上（Tauri 桥接） |
| **safe-area** | 不生效 | 顶部/底部留白避开状态栏和导航栏 |
| **数据库路径** | AppData 目录 | Android 内部存储 |
| **构建工具** | cargo + system deps | Gradle + Android NDK 交叉编译 |

### 移动端特有的前端代码

| 文件 | 移动端逻辑 |
|------|-----------|
| `ExportModal.vue` | `navigator.userAgent` 判断移动端，显示分享按钮 |
| `exportFile.ts` | 移动端走 `navigator.share()` 分享文件 |
| `shareContent.ts` | Tauri Share Plugin 调用 Android Intent |
| `FilterNav.vue` | 窗口宽度 ≤ 768px 时切换为底部弹出样式 |
| `App.vue` + `TopBar.vue` + `CameraModal.vue` + `ImageEditor.vue` | `safe-area-inset` 适配刘海屏/挖孔屏 |

---

## 🎯 关键设计决策

### 1. 为什么用 SQLite 而非其他数据库？

**决策**：本地存储使用 SQLite，通过 SeaORM 访问。

**理由**：
- **零配置**：用户无需安装数据库服务，开箱即用
- **单文件**：备份、迁移、同步都极为简单
- **嵌入式中等负载**：单用户场景 SQLite 性能绰绰有余
- **SeaORM** 提供了类型安全和迁移管理，未来切换到 PostgreSQL/MySQL 只需改连接字符串

### 2. SRS 为什么用 SDR 模型而非 SM-2？

**决策**：采用基于连续反馈的 SDR（Stability-Difficulty-Retrievability）模型。

**理由**：
- **连续反馈**：SM-2 只有 0-5 六个离散等级，SDR 支持 [0, 1] 连续值
- **自适应难度**：SDR 有独立的难度参数 $D$，会随历史反馈慢变
- **遗忘曲线拟合**：$R = e^{-t/S}$ 更符合记忆科学中的指数遗忘曲线
- **调参灵活**：所有参数（学习率、初始值、边界）都在 `config` 模块集中管理

详见 [SRS 算法文档](SRS_ALGORITHM.md)

### 3. 同步协议为什么设计为离线优先 + 握手模式？

**决策**：采用"离线优先 + 双向握手"的同步策略。

**理由**：
- **离线可用**：用户在地铁/无网络环境下可正常使用，有网时自动同步
- **版本向量**：每条记录有独立 version，握手时只需传轻量 header
- **冲突可解**：支持手动解决多设备同时修改同一记录的冲突
- **通用性强**：同一套协议适用于 7 张业务表

详见 [同步协议文档](SYNC_PROTOCOL.md)

### 4. 为什么用 Tauri 而非 Electron？

- **体积更小**：安装包约 10MB（Electron 约 150MB+）
- **性能更好**：Rust 后端比 Node.js 后端更高效
- **内存占用低**：Tauri 约 50MB，Electron 约 200MB+
- **安全性**：Rust 的内存安全保证 + Tauri 的权限模型

### 5. LLM 为什么设计为可配置的通用接口？

- 用户可以选择任意兼容 OpenAI API 的服务（OpenAI、DeepSeek、本地 Ollama 等）
- 不做供应商锁定
- 配置存储在 localStorage，不经过后端

---

## 📊 数据结构关系

```mermaid
erDiagram
    UserConfig ||--o{ Subject : "1 → N"
    Subject ||--o{ ErrorQuestion : "1 → N"
    ErrorQuestion ||--o| SRSData : "1 → 1"
    ErrorQuestion ||--o{ ErrorTags : "1 → N"
    ErrorQuestion ||--o{ Attachment : "1 → N"
    ErrorQuestion }o--|| Source : "N → 1"

    UserConfig {
        string username PK
        string email
    }
    Subject {
        string id PK
        string name
        string color
    }
    ErrorQuestion {
        string id PK
        string prompt
        string type
        string answer
        string analysis
        string error_note
    }
    Source {
        string id PK
        string book
        string chapter
        string knowledge
    }
    SRSData {
        string id PK
        string question_id FK
        float stability
        float difficulty
        float recall_rate
        int next_review_at
    }
    ErrorTags {
        string id PK
        string name
        string color
    }
    Attachment {
        string id PK
        string question_id FK
        string type
        string file_type
        string base64_data
    }
```

| 表 | 记录数级（单用户） | 说明 |
|----|-------------------|------|
| user_config | 1 | 单用户配置 |
| subjects | 10-50 | 科目 |
| error_questions | 100-5000 | 错题主表 |
| srs_data | = 错题数 | 一对一关系 |
| sources | 50-500 | 来源 |
| error_tags | 200-2000 | 错因标签（多对多） |
| attachments | 100-2000 | 图片附件 |

---

## 🔗 同步架构

同步功能是本项目最复杂的部分，详见 [同步协议文档](SYNC_PROTOCOL.md)。

核心设计要点：

```mermaid
sequenceDiagram
    participant A as 设备 A (SQLite)
    participant S as 同步服务器 (Flask)
    participant B as 设备 B (SQLite)

    Note over A: 数据更新 → pending
    A->>S: 握手
    S->>B: 握手
    Note over S: 比对版本
    B-->>S: 握手响应
    A->>S: 推送（携带数据）
    S->>S: version++
    S->>B: 拉取（转发数据）
    B-->>S: 拉取完成
    S-->>A: 推送确认
    B->>S: 推送（本地修改）
    S->>A: 拉取（转发）
    Note over S: 多轮握手直到空结果
```

### 关键特性

1. **每记录版本号**：每条数据有独立 `version` 字段，单调递增
2. **延迟冲突检测**：只有真正发生编辑冲突时才提示，而非锁表
3. **软删除**：删除标记传播到所有设备后才真正清理

---


## 📈 性能考虑

| 场景 | 当前方案 | 优化空间 |
|------|----------|----------|
| 图片存储 | base64/BLOB 存入 SQLite | 大文件可改为文件系统存储 |
| 大数据量查询 | 基础分页 (limit/offset) | 可加游标分页 |
| 复习队列 | 全量加载后排序 | 可加索引优化 next_review_at 查询 |
| 同步 | 全量握手 | 可增量握手 |

---

> 架构相关的问题或建议，请提交 [GitHub Issue](https://github.com/zpb911km/SmartErrorNotebook/issues)
