# Smart Error Notebook API 参考

> 本文档列出所有 Tauri `invoke` 命令及其参数、返回值，以及前后端数据契约。
>
> **平台说明**：以下所有 Tauri 命令在桌面端和 Android 端均可调用。
> 平台差异主要体现在**前端层**（如 Android 端使用 Web Share API 替代桌面端的保存对话框），
> 而非 Rust 后端层。详见 [架构设计 - 移动端架构](ARCHITECTURE.md#-移动端架构)。

---

## 📑 目录

1. [如何调用](#-如何调用)
2. [命令索引](#-命令索引)
3. [错题 (ErrorQuestion)](#-错题-errorquestion)
4. [科目 (Subject)](#-科目-subject)
5. [来源 (Source)](#-来源-source)
6. [错因标签 (ErrorTag)](#-错因标签-errortag)
7. [附件 (Attachment)](#-附件-attachment)
8. [SRS 复习数据](#-srs-复习数据)
9. [SRS 工具函数](#-srs-工具函数)
10. [同步 (Sync)](#-同步-sync)
11. [数据契约](#-数据契约)

---

## 💻 如何调用

所有 API 通过 Tauri 的 `invoke` 函数从前端调用：

```typescript
import { invoke } from '@tauri-apps/api/core'

// 无参数
const result = await invoke('command_name')

// 有参数（参数名使用 Rust 侧的 snake_case）
const result = await invoke('command_name', {
  param_name: value
})
```

> **注意**：所有参数名使用 **snake_case**（Rust 风格），而非 camelCase。

---

## 📋 命令索引

| 分类 | 命令 | 说明 |
|------|------|------|
| **错题** | `get_questions` | 获取错题列表 |
| | `get_question` | 获取单个错题 |
| | `create_question` | 创建错题 |
| | `update_question` | 更新错题 |
| | `delete_question` | 软删除错题 |
| | `get_question_stats` | 获取统计信息 |
| | `upsert_error_question` | 创建或更新（同步用） |
| **科目** | `get_subjects` | 获取所有科目 |
| | `create_subject` | 创建科目 |
| | `update_subject` | 更新科目 |
| | `delete_subject` | 删除科目 |
| | `upsert_subject` | 创建或更新（同步用） |
| **来源** | `get_sources` | 获取来源列表 |
| | `get_source` | 获取单个来源 |
| | `get_books` | 获取所有书名 |
| | `get_chapters` | 获取章节列表 |
| | `get_knowledges` | 获取知识点列表 |
| | `create_source` | 创建来源 |
| | `update_source` | 更新来源 |
| | `delete_source` | 删除来源 |
| | `get_or_create_source_id` | 获取或创建来源 ID |
| | `upsert_source` | 创建或更新（同步用） |
| **错因标签** | `get_error_tags` | 获取所有标签 |
| | `get_full_error_tags` | 获取完整标签信息 |
| | `get_error_tags_for_question` | 获取题目的标签 |
| | `create_error_tags_for_question` | 为题目创建标签 |
| | `delete_error_tag` | 删除标签 |
| | `update_error_tag_by_id` | 按 ID 更新标签 |
| | `update_error_tag_by_name` | 按名称更新标签 |
| | `upsert_error_tag` | 创建或更新（同步用） |
| **附件** | `create_attachment` | 创建附件 |
| | `create_attachments_for_question` | 为题目批量创建附件 |
| | `get_attachments_by_question` | 获取题目的附件列表 |
| | `delete_attachment` | 删除附件 |
| | `upsert_attachment` | 创建或更新（同步用） |
| **SRS** | `create_srs_data` | 初始化 SRS 数据 |
| | `get_due_questions` | 获取待复习题目 |
| | `submit_review_result` | 提交复习反馈 |
| | `get_question_srs_status` | 获取 SRS 状态 |
| | `reset_srs_progress` | 重置 SRS 进度 |
| | `upsert_srs_data` | 创建或更新（同步用） |
| **SRS 工具** | `get_due_count` | 获取待复习数量 |
| | `get_srs_statistics` | 获取 SRS 统计 |
| | `get_all_cards` | 获取所有 SRS 卡片 |
| **同步** | `get_all_pending_records` | 获取所有待同步记录 |
| | `get_record_for_upload` | 获取上传记录 |
| | `set_record_sync_status_version` | 设置同步状态和版本 |
| | `get_all_records` | 获取所有记录（握手用） |
| | `purge_synced_deletions` | 清理已同步的删除 |
| | `check_orphan_records` | 检查孤儿记录 |

---

## 📝 错题 (ErrorQuestion)

### `get_questions`

获取错题列表，支持筛选。

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `filter` | `QuestionFilter` | 否 | 筛选条件 |

**`QuestionFilter`：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `subject_id` | `string?` | 科目 ID |
| `search` | `string?` | 搜索关键词（题干、解析、笔记） |
| `limit` | `number?` | 返回数量限制 |
| `offset` | `number?` | 偏移量（分页） |

**返回：** `ErrorQuestion[]`

### `get_question`

获取单个错题详情。

**参数：** `{ id: string }`

**返回：** `ErrorQuestion`

### `create_question`

创建新错题。

**参数：** `{ input: Omit<ErrorQuestion, 'id'> }`

**返回：** `ErrorQuestion`

### `update_question`

更新错题（支持部分更新）。

**参数：** `{ input: UpdateQuestionInput }`

**`UpdateQuestionInput`：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | 错题 ID |
| `subject_id` | `string?` | 科目 ID |
| `source_id` | `string?` | 来源 ID |
| `prompt` | `string?` | 题干 |
| `type` | `string?` | 题型 |
| `answer` | `string?` | 标准答案 |
| `analysis` | `string?` | 解析 |
| `error_note` | `string?` | 错题笔记 |

**返回：** `ErrorQuestion`

### `delete_question`

软删除错题。

**参数：** `{ id: string }`

**返回：** `void`

### `get_question_stats`

**参数：** 无

**返回：** `{ total: number }`

---

## 📚 科目 (Subject)

### `get_subjects`

**参数：** 无

**返回：** `Subject[]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | UUID |
| `name` | `string` | 科目名称 |
| `color` | `string?` | 标识色 |

### `create_subject`

**参数：** `{ name: string, color?: string }`

**返回：** `Subject`

### `update_subject`

**参数：** `{ id: string, name?: string, color?: string }`

**返回：** `Subject`

### `delete_subject`

**参数：** `{ id: string }`

**返回：** `void`

---

## 📖 来源 (Source)

来源采用三级分类：`书 → 章节 → 知识点`

### `get_sources`

**参数：** `{ filter?: { subject_id?: string } }`

**返回：** `Source[]`

### `get_source`

**参数：** `{ id: string }`

**返回：** `Source`

### `get_books`

获取指定科目的所有书名。

**参数：** `{ subject_id?: string }`

**返回：** `string[]`

### `get_chapters`

获取指定书名的所有章节。

**参数：** `{ subject_id?: string, book?: string }`

**返回：** `string[]`

### `get_knowledges`

获取指定章节的所有知识点。

**参数：** `{ subject_id?: string, book?: string, chapter?: string }`

**返回：** `string[]`

### `create_source`

**参数：** `{ input: { subject_id?: string, book?: string, chapter?: string, knowledge?: string } }`

**返回：** `Source`

### `get_or_create_source_id`

根据三级分类查找或自动创建来源，返回 ID。

**参数：** `{ subject_id?: string, book?: string, chapter?: string, knowledge?: string }`

**返回：** `string` (Source ID)

---

## 🏷️ 错因标签 (ErrorTag)

### `get_error_tags`

**参数：** 无

**返回：** `string[]`（标签名称列表）

### `get_full_error_tags`

**参数：** 无

**返回：** `ErrorTags[]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | UUID |
| `question_id` | `string` | 关联错题 ID |
| `name` | `string` | 标签名称 |
| `color` | `string` | 标签颜色 |

### `get_error_tags_for_question`

**参数：** `{ question_id: string }`

**返回：** `ErrorTags[]`

### `create_error_tags_for_question`

为题目批量设置标签。

**参数：** `{ question_id: string, tags: Array<{ name: string, color: string }> }`

**返回：** `ErrorTags[]`

### `delete_error_tag`

**参数：** `{ id: string }`

**返回：** `void`

### `update_error_tag_by_id`

**参数：** `{ id: string, name?: string, color?: string }`

**返回：** `ErrorTags`

### `update_error_tag_by_name`

**参数：** `{ name: string, new_name?: string, color?: string }`

**返回：** `ErrorTags`

---

## 🖼️ 附件 (Attachment)

### `create_attachment`

**参数：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `question_id` | `string` | 错题 ID |
| `type_` | `string` | 附件类型 (`original` / `answer`) |
| `file_type` | `string` | 文件类型 (`img`) |
| `base64_data` | `string` | base64 编码的数据 |

**返回：** `Attachment`

### `create_attachments_for_question`

批量创建附件。

**参数：** `{ question_id: string, attachments: CreateAttachmentInput[] }`

**返回：** `Attachment[]`

### `get_attachments_by_question`

**参数：** `{ question_id: string }`

**返回：** `Attachment[]`

### `delete_attachment`

**参数：** `{ id: string }`

**返回：** `void`

---

## 🧠 SRS 复习数据

### `create_srs_data`

为题目初始化 SRS 数据。

**参数：** `{ input: { question_id: string, difficulty?: number } }`

**返回：** `SRSData`

### `get_due_questions`

获取当前到期的待复习题目。

**参数：** `{ subject_id?: string, now?: number }`

**返回：** `Array<{ question: ErrorQuestion, srs: SRSData }>`

### `submit_review_result`

提交一次复习的结果。

**参数：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `question_id` | `string` | 错题 ID |
| `feedback` | `number` | 反馈值 [0, 1] |

**返回：** `ReviewOutput`

| 字段 | 类型 | 说明 |
|------|------|------|
| `next_interval_days` | `number` | 下次复习间隔（天） |
| `new_stability` | `number` | 更新后的稳定性 |
| `new_difficulty` | `number` | 更新后的难度 |
| `next_review_at` | `number` | 下次复习时间戳（秒） |

### `get_question_srs_status`

**参数：** `{ question_id: string }`

**返回：** `SRSCardOutput`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | SRS 数据 ID |
| `question_id` | `string` | 错题 ID |
| `stability` | `number` | 稳定性（天） |
| `difficulty` | `number` | 难度 [1, 10] |
| `recall_rate` | `number` | 预测召回率 |
| `next_review_at` | `number?` | 下次复习时间戳 |
| `last_review_at` | `number?` | 上次复习时间戳 |
| `review_count` | `number` | 复习次数 |
| `is_due` | `boolean` | 是否到期 |

### `reset_srs_progress`

重置指定题目的 SRS 进度。

**参数：** `{ question_id: string }`

**返回：** `void`

---

## 📊 SRS 工具函数

### `get_due_count`

**参数：** `{ subject_id?: string }`

**返回：** `number`

### `get_srs_statistics`

**参数：** 无

**返回：** `SRSStatistics`

| 字段 | 类型 | 说明 |
|------|------|------|
| `total` | `number` | 总卡片数 |
| `due_count` | `number` | 待复习数量 |
| `new_cards` | `number` | 新卡片数量 |
| `avg_stability` | `number` | 平均稳定性（天） |
| `avg_difficulty` | `number` | 平均难度 |
| `total_reviews` | `number` | 总复习次数 |

### `get_all_cards`

**参数：** 无

**返回：** `SRSCardOutput[]`

---

## 🔄 同步 (Sync)

### `get_all_pending_records`

获取所有状态为 `pending` 的记录（待同步）。

**参数：** 无

**返回：** `SyncRecordHeader[]`

### `get_record_for_upload`

获取指定 ID 的完整记录数据（含 data 负载）用于上传。

**参数：** `{ id: string, table_name: string }`

**返回：** `SyncRecord`

### `set_record_sync_status_version`

更新记录的同步状态和版本号。

**参数：** `{ id: string, table_name: string, version: number, status: string }`

**返回：** `void`

### `get_all_records`

获取所有记录的头信息（握手协议用）。

**参数：** 无

**返回：** `SyncRecordHeader[]`

### `purge_synced_deletions`

清理已同步到服务器的删除标记。

**参数：** 无

**返回：** `void`

### `check_orphan_records`

检查并清理孤儿记录（子表记录已删但父表不存在的记录）。

**参数：** 无

**返回：** `{ orphan_records_soft_deleted: string[], total_checked: number }`

---

## 📐 数据契约

### ErrorQuestion 实体

```typescript
interface ErrorQuestion {
  id: string              // UUID
  user_id: string         // 用户 ID ⚠️ 后端字段名: userid
  subject_id: string      // 科目 ID ⚠️ 后端字段名: subjectid
  source_id?: string      // 来源 ID ⚠️ 后端字段名: sourceid
  prompt: string          // 题干 (支持 Markdown + LaTeX)
  type: QuestionType      // 题型 ⚠️ 后端字段名: type_
  answer?: string         // 标准答案
  analysis?: string       // 解析
  error_note?: string     // 错题笔记
}
```

### SRSData 实体

```typescript
interface SRSData {
  id: string                  // UUID
  question_id: string         // 关联错题 ID
  stability: number           // 稳定性（天）
  difficulty: number          // 难度 [1.0, 10.0]
  recall_rate: number         // 预测召回率
  next_review_at: number|null // 下次复习时间戳（秒）
  last_review_at: number|null // 上次复习时间戳 ⚠️ 后端: lastreviewed_at
  review_count: number        // 复习次数
  feedback_history: string    // 最近 5 次反馈 JSON
}
```

### Source 实体

```typescript
interface Source {
  id: string
  question_id?: string
  subject_id?: string
  book?: string        // 书名
  chapter?: string     // 章节
  knowledge?: string   // 知识点
  subject?: Subject    // 关联科目信息
}
```

### Attachment 实体

```typescript
interface Attachment {
  id: string
  question_id: string
  type: string          // 'original' | 'answer' ⚠️ 后端: type_
  file_type: string     // 'img'
  base64_data: string   // base64 图片数据
  hash: string          // 文件哈希
}
```

> API 相关问题请提交 [GitHub Issue](https://github.com/zpb911km/SmartErrorNotebook/issues)
