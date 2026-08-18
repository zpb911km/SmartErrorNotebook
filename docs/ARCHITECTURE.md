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

本项目当前采用 **三层架构**：前端展示层 → Rust 业务逻辑层 → 本地存储层。
远程同步和社区已停用；`server/` 是不兼容当前数据库模型的历史实现，不属于受支持架构。

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
            V9[Sync 停用说明]
        end

        subgraph C["Components（18 组件）"]
            C1[TopBar / BottomNav]
            C2[CameraModal / ImageEditor]
            C3[MarkdownTextarea / ErrorTagSelector]
            C4[Card]
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

        subgraph Cmd["command/legacy（兼容 Tauri 命令）"]
            CM1[subject.rs] --- CM2[error_question.rs]
            CM3[error_tag.rs] --- CM4[source.rs]
            CM5[attachment.rs] --- CM6[srs_data.rs]
            CM7[sync.rs] --- CM8[file.rs]
        end

        subgraph Repo["repository/ + data/repository/"]
            RP1[Repository traits]
            RP2[legacy 兼容模型]
            RP3[SeaORM 事务实现]
        end

        subgraph DAL["data/database/ + crates/migration/"]
            D1[connection.rs 数据库连接]
            D2[entity/ 规范化 ORM 实体]
            D3[migration 独立 crate]
        end

        subgraph SRS["srs/（SDR 算法引擎）"]
            R1[mod.rs]
            R2[review_card 复习核心]
            R3[predict_retrievability 预测]
            R4[compute_next_interval 间隔]
        end

        Cmd --> Repo
        Repo --> DAL
        SRS --> Cmd
    end

    %% ══════════ 本地存储层 ══════════
    Backend -->|SQLite 文件| Storage

    subgraph Storage["💾 本地存储层 (SQLite)"]
        T1[subject 科目]
        T2[source 来源]
        T3[question 错题]
        T4[srs_data 复习数据]
        T5[attachment 附件]
        T6[tag 错因标签]
        T7[question_attachment_cross_ref]
        T8[question_tag_cross_ref]
    end

    Storage -.-> Location
    Location[数据库位置按平台:<br/>Win: %APPDATA%/<br/>Mac: ~/Library/<br/>Linux: ~/.local/share/]

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
| `command/` | 命令注册及 legacy 兼容处理器 | 函数标注 `#[tauri::command]`，统一在 `command/mod.rs` 注册 |
| `repository/` | 仓储 traits 与 legacy 契约 | Command 通过事务中的 RepositoryFactory 访问持久化数据 |
| `data/repository/` | SeaORM 仓储实现 | 所有命令写入由 RepositoryTransactionExecutor 包裹 |
| `data/database/entity/` | 规范化 SeaORM 实体 | 使用 UUID、DateTime 和交叉引用实体 |
| `model/` | 内部领域模型 | 与 legacy IPC 请求/响应隔离 |
| `crates/migration/` | 独立迁移 crate | 命名 `mYYYYMMDD_NNNNNN_desc.rs` |
| `srs/` | 核心复习算法 | 纯函数，不依赖 Tauri/数据库 |

### 历史同步服务器 (`server/`)

此目录仅供历史参考，客户端不会连接，当前开发和发布流程不支持部署。

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

### 3. 未来同步协议有哪些底线？

当前没有受支持的远程同步实现。未来协议必须使用 `(table_name, id)` 复合标识，
完整同步附件/标签的 `question_ids` 关系集合，并由服务端生成单调递增版本。
详见 [同步协议草案](SYNC_PROTOCOL.md)。

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
    Subject ||--o{ Source : "1 → N"
    Source ||--o{ Question : "1 → N"
    Question ||--o| SRSData : "1 → 1"
    Question ||--o{ QuestionAttachment : "1 → N"
    Attachment ||--o{ QuestionAttachment : "1 → N"
    Question ||--o{ QuestionTag : "1 → N"
    Tag ||--o{ QuestionTag : "1 → N"

    Subject {
        uuid id PK
        string name
        string color
    }
    Source {
        uuid id PK
        uuid subject_id FK
        string book
        string chapter
        string knowledge
    }
    Question {
        uuid id PK
        uuid source_id FK
        string stem
        string question_type
        string correct_answer
        string explanation
        string note
    }
    SRSData {
        uuid question_id PK
        float stability
        float difficulty
        datetime next_review_at
    }
    Tag {
        uuid id PK
        string name
        string color
    }
    Attachment {
        uuid id PK
        string mime_type
        blob data
        string sha256
    }
    QuestionAttachment {
        uuid question_id PK,FK
        uuid attachment_id PK,FK
    }
    QuestionTag {
        uuid question_id PK,FK
        uuid tag_id PK,FK
    }
```

| 表 | 记录数级（单用户） | 说明 |
|----|-------------------|------|
| subject | 10-50 | 科目 |
| source | 50-500 | 来源及科目归属 |
| question | 100-5000 | 错题主表 |
| srs_data | = 错题数 | 一对一关系 |
| tag | 200-2000 | 可复用错因标签 |
| attachment | 100-2000 | 二进制附件 |
| question_tag_cross_ref | 随关系增长 | 题目与标签多对多关系 |
| question_attachment_cross_ref | 随关系增长 | 题目与附件多对多关系 |

---

## 🔗 同步架构

当前客户端不执行远程同步。仅保留本地同步元数据维护命令和未来协议所需的数据字段。
`server/` 不受支持，不能据此推断当前产品具备同步能力。未来设计约束见
[同步协议草案](SYNC_PROTOCOL.md)。

## 📈 性能考虑

| 场景 | 当前方案 | 优化空间 |
|------|----------|----------|
| 图片存储 | base64/BLOB 存入 SQLite | 大文件可改为文件系统存储 |
| 大数据量查询 | 基础分页 (limit/offset) | 可加游标分页 |
| 复习队列 | 全量加载后排序 | 可加索引优化 next_review_at 查询 |
| 远程同步 | 当前停用 | 按协议草案重新实现 |

---

> 架构相关的问题或建议，请提交 [GitHub Issue](https://github.com/zpb911km/SmartErrorNotebook/issues)
