# API Reference

Smart Error Notebook 通过 Tauri `invoke` 暴露 50 个数据库命令。本参考手册描述命令签名、参数、返回值、错误和可观察行为。

生产实现以 `src-tauri/src/command/mod.rs` 的命令注册表及 `src-tauri/src/command/legacy/` 中的兼容处理器为准。数据库连接、仓储实现和迁移器不属于 IPC API。

## 快速开始

```ts
import { invoke } from '@tauri-apps/api/core'

const questions = await invoke<ErrorQuestion[]>('legacy_get_questions', {
  filter: { subject_id: 'subject-id', limit: 20 },
})

const status = await invoke<SRSCardOutput | null>('legacy_get_question_srs_status', {
  questionId: 'question-id',
})
```

### 参数命名

- 命令顶层参数采用 Tauri 默认的 `camelCase`，例如 `questionId`、`recordId`、`tagId` 和 `subjectId`。
- `input`、`filter` 和数组元素是 Serde 结构，内部字段使用本文列出的名称，通常为 `snake_case`。
- 错题创建和更新输入使用 `type`；`ErrorQuestion` 输出使用 `type_`。
- 可选顶层参数可以省略或传 `null`。必填字段缺失、命名错误或类型错误会在 command 执行前产生反序列化错误。

下文的“签名”使用 TypeScript 风格的类型伪代码描述参数形状，并非可直接执行的对象字面量；“快速开始”中的代码才是完整调用示例。

### 通用写入行为

- 时间字段为 UTC Unix 秒；本地生成的 ID 为 UUID v4。
- 本地创建默认写入 `version = 0`、`sync_status = "pending"`、`sync_hash = null`。
- 本地更新和软删除写入 `sync_status = "pending"`，当前不会递增 `version`。
- 所有同步 upsert 都忽略输入的 `status`，并写入 `sync_status = "synced"`。
- 除非命令说明明确给出排序规则，否则集合顺序不属于契约。

## 命令索引

| 分组 | 命令 |
| --- | --- |
| 科目 | `legacy_get_subjects`, `legacy_create_subject`, `legacy_update_subject`, `legacy_delete_subject`, `legacy_upsert_subject` |
| 错题 | `legacy_get_questions`, `legacy_get_question`, `legacy_create_question`, `legacy_update_question`, `legacy_delete_question`, `legacy_get_question_stats`, `legacy_upsert_error_question` |
| 来源 | `legacy_get_sources`, `legacy_get_source`, `legacy_get_books`, `legacy_get_chapters`, `legacy_get_knowledges`, `legacy_create_source`, `legacy_update_source`, `legacy_delete_source`, `legacy_get_or_create_source_id`, `legacy_upsert_source` |
| 错因标签 | `legacy_create_error_tags_for_question`, `legacy_get_error_tags`, `legacy_get_full_error_tags`, `legacy_get_error_tags_for_question`, `legacy_delete_error_tag`, `legacy_update_error_tag_by_name`, `legacy_update_error_tag_by_id`, `legacy_upsert_error_tag` |
| 附件 | `legacy_create_attachment`, `legacy_create_attachments_for_question`, `legacy_get_attachments_by_question`, `legacy_delete_attachment`, `legacy_upsert_attachment` |
| SRS | `legacy_create_srs_data`, `legacy_get_due_questions`, `legacy_submit_review_result`, `legacy_get_question_srs_status`, `legacy_reset_srs_progress`, `legacy_get_due_count`, `legacy_get_srs_statistics`, `legacy_get_all_cards`, `legacy_upsert_srs_data` |
| 同步 | `legacy_get_all_records`, `legacy_get_all_pending_records`, `legacy_get_record_for_upload`, `legacy_set_record_sync_status_version`, `legacy_purge_synced_deletions`, `legacy_check_orphan_records` |

## 数据类型

### 同步元数据

```ts
interface SyncMetadata {
  created_at: number
  updated_at: number
  deleted_at: number | null
  version: number
  sync_status: string
  sync_hash: string | null
}
```

### `Subject`

```ts
interface Subject extends SyncMetadata {
  id: string
  name: string
  color: string | null
}
```

### `ErrorQuestion`

```ts
interface ErrorQuestion extends SyncMetadata {
  id: string
  userid: string
  subjectid: string
  sourceid: string | null
  prompt: string
  type_: string
  answer: string | null
  analysis: string | null
  error_note: string | null
}
```

### `Source`

```ts
interface Source extends SyncMetadata {
  id: string
  question_id: string | null
  subject_id: string | null
  book: string | null
  chapter: string | null
  knowledge: string | null
}
```

### `ErrorTag`

```ts
interface ErrorTag extends SyncMetadata {
  id: string
  question_id: string
  name: string
  color: string
}
```

### `Attachment`

```ts
interface Attachment {
  id: string
  question_id: string
  type_: string
  file_type: string
  base64_data: string
  hash: string
}
```

`Attachment` 是专用输出，不包含同步元数据。数据库字节无法解码为 UTF-8 时，`base64_data` 返回空字符串。

### SRS 类型

```ts
interface SRSCardOutput {
  id: string
  question_id: string
  stability: number
  difficulty: number
  recall_rate: number
  next_review_at: number | null
  last_review_at: number | null
  review_count: number
  is_due: boolean
}

interface ReviewOutput {
  next_interval_days: number
  new_stability: number
  new_difficulty: number
  next_review_at: number
}

interface SRSStatistics {
  total: number
  due_count: number
  new_cards: number
  avg_stability: number
  avg_difficulty: number
  total_reviews: number
}
```

### 同步类型

```ts
interface SyncRecordHeader {
  id: string
  table_name: string
  version: number
  status: string
  deleted_at: number | null
  updated_at: number
  created_at: number
}

interface SyncRecord extends Omit<SyncRecordHeader, 'created_at'> {
  data: Record<string, unknown>
}

interface OrphanCheckResult {
  orphan_records_soft_deleted: string[]
  total_checked: number
}
```

## 科目

### `legacy_get_subjects`

**签名**

```text
invoke<Subject[]>('legacy_get_subjects')
```

**返回**：所有未软删除科目。

**行为**：排除 `deleted_at IS NOT NULL` 的记录；顺序不保证。

### `legacy_create_subject`

**签名**

```text
invoke<Subject>('legacy_create_subject', {
  input: { name: string, color?: string | null },
})
```

**返回**：新建的 `Subject`。

### `legacy_update_subject`

**签名**

```text
invoke<Subject>('legacy_update_subject', {
  input: { id: string, name?: string | null, color?: string | null },
})
```

**行为**：`name` 或 `color` 为 `null`/省略时保留原值；当前接口不能清空已有颜色。

**错误**：ID 不存在时返回 `Subject not found`。

### `legacy_delete_subject`

**签名**：`invoke<null>('legacy_delete_subject', { id: string })`

**行为**：软删除科目并标记 `pending`；不会级联处理题目或来源。

**错误**：ID 不存在时返回 `Subject not found`。

### `legacy_upsert_subject`

**签名**

```text
invoke<null>('legacy_upsert_subject', {
  input: {
    id: string
    version: number
    status: string
    deleted_at?: number | null
    name: string
    color?: string | null
  },
})
```

**行为**：按 ID 插入或覆盖业务字段、版本和删除时间；`status` 被忽略，保存状态固定为 `synced`。

## 错题

### `legacy_get_questions`

**签名**

```text
invoke<ErrorQuestion[]>('legacy_get_questions', {
  filter?: {
    subject_id?: string
    search?: string
    limit?: number
    offset?: number
  } | null,
})
```

**行为**：

- 仅返回未软删除记录。
- `subject_id` 精确匹配。
- `search` 以 `%keyword%` 匹配 `prompt`、`analysis` 和 `error_note`。
- 结果按 `updated_at` 降序，再应用 `limit` 和 `offset`。

### `legacy_get_question`

**签名**：`invoke<ErrorQuestion>('legacy_get_question', { id: string })`

**行为**：按主键读取，不过滤软删除记录。

**错误**：ID 不存在时返回 `Question not found`。

### `legacy_create_question`

**签名**

```text
invoke<ErrorQuestion>('legacy_create_question', {
  input: {
    user_id: string
    subject_id: string
    source_id?: string | null
    prompt: string
    type: string
    answer?: string | null
    analysis?: string | null
    error_note?: string | null
  },
})
```

**行为**：当前不验证科目或来源是否存在。

### `legacy_update_question`

**签名**

```text
invoke<ErrorQuestion>('legacy_update_question', {
  input: {
    id: string
    subject_id?: string | null
    source_id?: string | null
    prompt?: string | null
    type?: string | null
    answer?: string | null
    analysis?: string | null
    error_note?: string | null
  },
})
```

**行为**：

- 修改 `subject_id` 时验证目标科目存在。
- `source_id` 也接受别名 `sourceid`。
- 可选字段为 `null`/省略时保留原值，因此不能借此清空来源、答案、解析或错题笔记。

**错误**：题目不存在时返回 `Question not found`；目标科目不存在时返回 `Subject not found`。

### `legacy_delete_question`

**签名**：`invoke<null>('legacy_delete_question', { id: string })`

**行为**：操作在同一事务内完成：软删除题目和对应的 SRS 记录，并解除题目的全部附件、标签关系。解除关系后仍被其他题目引用的附件或标签会保留并标记为 `pending`；失去最后一条关系的资源会同时软删除。依赖清理先于题目存在性检查。

**错误**：题目不存在时返回 `Question not found`。

### `legacy_get_question_stats`

**签名**：`invoke<{ total: number }>('legacy_get_question_stats')`

**返回**：未软删除题目总数。

### `legacy_upsert_error_question`

**签名**

```text
invoke<null>('legacy_upsert_error_question', {
  input: {
    id: string
    version: number
    status: string
    deleted_at?: number | null
    userid: string
    subjectid: string
    sourceid?: string | null
    prompt: string
    type_: string
    answer?: string | null
    analysis?: string | null
    error_note?: string | null
    sync_hash?: string | null
  },
})
```

**行为**：

- `subjectid`/`sourceid` 也接受 `subject_id`/`source_id`。
- 插入时写入全部字段；更新已有记录时不修改 `userid` 和 `sync_hash`。
- `status` 被忽略，保存状态固定为 `synced`。

## 来源

### `legacy_get_sources`

**签名**：`invoke<Source[]>('legacy_get_sources', { filter?: { subject_id?: string } | null })`

**行为**：排除软删除记录；提供 `subject_id` 时精确过滤。

### `legacy_get_source`

**签名**：`invoke<Source>('legacy_get_source', { id: string })`

**行为**：不排除软删除记录。

**错误**：ID 不存在时返回 `Source not found`。

### `legacy_get_books`

**签名**：`invoke<string[]>('legacy_get_books', { subjectId?: string | null })`

**返回**：未软删除记录中的非空书名去重集合；顺序不保证。

### `legacy_get_chapters`

**签名**：`invoke<string[]>('legacy_get_chapters', { subjectId?: string | null, book: string })`

**返回**：匹配书名和可选科目的非空章节去重集合；顺序不保证。

### `legacy_get_knowledges`

**签名**：`invoke<string[]>('legacy_get_knowledges', { subjectId?: string | null, book: string, chapter: string })`

**返回**：匹配书名、章节和可选科目的非空知识点去重集合；顺序不保证。

### `legacy_create_source`

**签名**

```text
invoke<Source>('legacy_create_source', {
  input: {
    subject_id?: string | null
    book?: string | null
    chapter?: string | null
    knowledge?: string | null
  },
})
```

**行为**：新记录的 `question_id = null`；不验证科目是否存在。

### `legacy_update_source`

**签名**

```text
invoke<Source>('legacy_update_source', {
  input: {
    id: string
    subject_id?: string | null
    book?: string | null
    chapter?: string | null
    knowledge?: string | null
  },
})
```

**行为**：仅更新非 `null` 字段，当前不能清空已有字段。

**错误**：ID 不存在时返回 `Source not found`。

### `legacy_delete_source`

**签名**：`invoke<null>('legacy_delete_source', { id: string })`

**行为**：软删除来源并标记 `pending`。

**错误**：ID 不存在时返回 `Source not found`。

### `legacy_get_or_create_source_id`

**签名**

```text
invoke<string>('legacy_get_or_create_source_id', {
  input: {
    subject_id?: string | null
    book?: string | null
    chapter?: string | null
    knowledge?: string | null
  },
})
```

**行为**：在未软删除记录中对四个字段进行包括 `NULL` 在内的精确匹配；找到时返回已有 ID，否则创建来源并返回新 ID。

### `legacy_upsert_source`

**签名**

```text
invoke<null>('legacy_upsert_source', {
  input: {
    id: string
    version: number
    status: string
    deleted_at?: number | null
    question_id?: string | null
    subject_id?: string | null
    book?: string | null
    chapter?: string | null
    knowledge?: string | null
  },
})
```

**行为**：按 ID 插入或覆盖；`status` 被忽略，保存状态固定为 `synced`。

## 错因标签

### `legacy_create_error_tags_for_question`

**签名**

```text
invoke<ErrorTag[]>('legacy_create_error_tags_for_question', {
  input: {
    question_id: string
    tags: Array<{ name: string, color: string }>
  },
})
```

**行为**：按输入顺序逐条创建，不显式验证题目存在；空数组返回空数组。整批写入位于同一事务中，任一条失败时不会提交部分结果。

### `legacy_get_error_tags`

**签名**：`invoke<ErrorTag[]>('legacy_get_error_tags')`

**返回**：未软删除标签按 `name` 去重后的集合。

**行为**：顺序及同名标签中具体保留哪条记录不保证。

### `legacy_get_full_error_tags`

**签名**：`invoke<ErrorTag[]>('legacy_get_full_error_tags')`

**返回**：所有未软删除标签，不去重。

### `legacy_get_error_tags_for_question`

**签名**：`invoke<ErrorTag[]>('legacy_get_error_tags_for_question', { questionId: string })`

**返回**：指定题目的全部未软删除标签。

### `legacy_delete_error_tag`

**签名**：`invoke<null>('legacy_delete_error_tag', { tagId: string, questionId: string })`

**行为**：解除标签与指定题目的关系；只有最后一条关系解除后才软删除标签并标记 `pending`。

**错误**：关系不存在时返回错误。

### `legacy_update_error_tag_by_name`

**签名**

```text
invoke<null>('legacy_update_error_tag_by_name', {
  oldName: string
  newName: string
  newColor: string
})
```

**行为**：更新所有未软删除且名称等于 `oldName` 的记录；无匹配时仍成功。

### `legacy_update_error_tag_by_id`

**签名**

```text
invoke<null>('legacy_update_error_tag_by_id', {
  tagId: string
  newTagName: string
  newTagColor?: string | null
})
```

**行为**：`newTagColor = null` 时保留原颜色。

**错误**：ID 不存在或记录已软删除时返回 `标签不存在`。

### `legacy_upsert_error_tag`

**签名**

```text
invoke<null>('legacy_upsert_error_tag', {
  input: {
    id: string
    version: number
    status: string
    deleted_at?: number | null
    question_ids: string[]
    question_id?: string
    name: string
    color: string
  },
})
```

**行为**：按 ID 插入或覆盖，并将关系精确更新为去重后的题目 ID 集合。`question_id` 仅用于兼容旧的单关系调用；`status` 被忽略，保存状态固定为 `synced`。未删除记录的关系集合不能为空。

## 附件

### `legacy_create_attachment`

**签名**

```text
invoke<Attachment>('legacy_create_attachment', {
  input: {
    question_id: string
    type_: string
    file_type: string
    base64_data: string
  },
})
```

**行为**：将字符串的 UTF-8 字节写入数据库；`hash` 是生成 UUID 的前八个字符；不验证题目存在。

**已知限制**：调试日志直接读取 `base64_data[..100]`。输入少于 100 字节，或第 100 字节不是 UTF-8 字符边界时，当前实现会 panic。

### `legacy_create_attachments_for_question`

**签名**

```text
invoke<Attachment[]>('legacy_create_attachments_for_question', {
  questionId: string
  attachments: Array<{
    question_id: string
    type_: string
    file_type: string
    base64_data: string
  }>
})
```

**行为**：用顶层 `questionId` 覆盖每个元素的 `question_id`，再按顺序逐条创建。整批写入位于同一事务中，任一条失败时不会提交部分结果。

### `legacy_get_attachments_by_question`

**签名**：`invoke<Attachment[]>('legacy_get_attachments_by_question', { questionId: string })`

**返回**：指定题目的全部未软删除附件。

### `legacy_delete_attachment`

**签名**：`invoke<null>('legacy_delete_attachment', { id: string, questionId: string })`

**行为**：解除附件与指定题目的关系；只有最后一条关系解除后才软删除附件并标记 `pending`。

**错误**：关系不存在时返回错误。

### `legacy_upsert_attachment`

**签名**

```text
invoke<null>('legacy_upsert_attachment', {
  input: {
    id: string
    version: number
    status: string
    deleted_at?: number | null
    question_ids: string[]
    question_id?: string
    type_: string
    file_type: string
    base64_data: number[]
    hash: string
  },
})
```

**行为**：同步输入的 `base64_data` 是字节数组；按 ID 插入或覆盖，并将关系精确更新为去重后的题目 ID 集合。`question_id` 仅用于兼容旧的单关系调用；保存状态固定为 `synced`。未删除记录的关系集合不能为空。

## SRS

### `legacy_create_srs_data`

**签名**

```text
invoke<SRSCardOutput>('legacy_create_srs_data', {
  input: { question_id: string, difficulty?: number | null },
})
```

**行为**：不验证题目存在；使用算法初始稳定性和可选难度，`review_count = 1`，下次复习约为一天后。

**错误**：任意同 `question_id` 记录（包括软删除记录）已存在时返回 `SrsData is exist: <question_id>`。

### `legacy_get_due_questions`

**签名**：`invoke<SRSCardOutput[]>('legacy_get_due_questions', { limit?: number | null })`

**行为**：排除软删除记录；`next_review_at = null` 或时间已到视为到期；按稳定性升序，默认最多返回 1000 条。

### `legacy_submit_review_result`

**签名**

```text
invoke<ReviewOutput>('legacy_submit_review_result', {
  input: { question_id: string, feedback: number },
})
```

**行为**：

- `feedback` 必须位于 `[0, 1]`。
- 更新算法状态、复习次数、最近五次反馈、时间和 `pending` 状态。
- 查询现有记录时不排除软删除；成功提交会清空 `deleted_at`。

**错误**：记录不存在时返回 `SRS data not found`；反馈越界时返回 `Feedback must be in [0, 1], got <value>`。

### `legacy_get_question_srs_status`

**签名**：`invoke<SRSCardOutput | null>('legacy_get_question_srs_status', { questionId: string })`

**返回**：题目的未软删除 SRS 状态；不存在时返回 `null`。

### `legacy_reset_srs_progress`

**签名**：`invoke<SRSCardOutput>('legacy_reset_srs_progress', { questionId: string })`

**行为**：查询包括软删除记录在内的现有 SRS。记录存在时恢复初始参数、清除 `deleted_at` 并立即到期；不存在时创建立即到期的新记录。

### `legacy_get_due_count`

**签名**：`invoke<number>('legacy_get_due_count')`

**返回**：未软删除且已到期的记录数。

### `legacy_get_srs_statistics`

**签名**：`invoke<SRSStatistics>('legacy_get_srs_statistics')`

**行为**：仅统计未软删除记录；`new_cards` 是 `review_count === 1` 的数量；无记录时两个平均值均为零。

### `legacy_get_all_cards`

**签名**：`invoke<SRSCardOutput[]>('legacy_get_all_cards')`

**返回**：全部未软删除卡片；顺序不保证。

### `legacy_upsert_srs_data`

**签名**

```text
invoke<null>('legacy_upsert_srs_data', {
  input: {
    id: string
    version: number
    status: string
    deleted_at?: number | null
    question_id: string
    stability: number
    difficulty: number
    next_review_at?: number | null
    lastreviewed_at?: number | null
    review_count: number
    feedback_history: string
  },
})
```

**行为**：插入和更新都会应用传入的 `deleted_at`，保存状态固定为 `synced`。规范化存储使用 `question_id` 作为 SRS 记录标识，兼容字段 `id` 不参与持久化定位；输出中的 `id` 同样等于 `question_id`。

## 同步

同步聚合按以下固定表顺序执行：

1. `error_questions`
2. `subjects`
3. `srs_data`
4. `attachments`
5. `error_tags`
6. `sources`

表内顺序不保证。

### `legacy_get_all_records`

**签名**：`invoke<SyncRecordHeader[]>('legacy_get_all_records')`

**返回**：六张表的全部记录头，包括软删除和任意同步状态；不包含 `data`。

### `legacy_get_all_pending_records`

**签名**：`invoke<SyncRecord[]>('legacy_get_all_pending_records')`

**返回**：六张表中 `sync_status = "pending"` 的记录，包括 pending 的软删除记录。

**数据裁剪**：`data` 保留业务字段、`id` 和 `sync_hash`，移除 `version`、`sync_status`、`deleted_at`、`created_at` 和 `updated_at`。

### `legacy_get_record_for_upload`

**签名**：`invoke<SyncRecord>('legacy_get_record_for_upload', { recordId: string })`

**行为**：按固定表顺序使用 ID 查找；不同表存在相同 ID 时返回顺序靠前的记录。

**错误**：不存在时返回 `Record not found with id: <recordId>`。

### `legacy_set_record_sync_status_version`

**签名**

```text
invoke<string>('legacy_set_record_sync_status_version', {
  recordId: string
  status: 'pending' | 'synced' | 'conflict'
  version: number
})
```

**行为**：按固定表顺序更新第一条匹配记录的状态和版本，返回包含记录 ID 的确认字符串。状态输入不区分大小写，响应及其他接口统一输出小写。

**错误**：不存在时返回 `Record not found with id: <recordId>`；未知状态在命令执行前返回 `Unknown sync status: <status>`，不会修改记录。

### `legacy_purge_synced_deletions`

**签名**

```text
invoke<Record<string, { deleted: number }>>('legacy_purge_synced_deletions')
```

**行为**：对六张表物理删除同时满足 `sync_status = "synced"` 和 `deleted_at IS NOT NULL` 的记录。

**返回示例**

```json
{
  "error_questions": { "deleted": 1 },
  "subjects": { "deleted": 0 },
  "srs_data": { "deleted": 0 },
  "attachments": { "deleted": 0 },
  "error_tags": { "deleted": 0 },
  "sources": { "deleted": 0 }
}
```

六张表的清理在同一个仓储事务中执行。

### `legacy_check_orphan_records`

**签名**：`invoke<OrphanCheckResult>('legacy_check_orphan_records')`

**行为**：只检查当前未软删除记录。

| 记录 | 父记录缺失时的处理 | 报告格式 |
| --- | --- | --- |
| `error_tags` | 没有任何题目关系时软删除并标记 `pending` | `error_tags:<id>` |
| `attachments` | 没有任何题目关系时软删除并标记 `pending` | `attachments:<id>` |

`total_checked` 是上述两类活动记录的检查总数。题目、来源和 SRS 引用完整性由规范化数据库关系及写入事务维护，此命令不扫描这些实体。

## 契约测试

测试通过 Tauri mock runtime 调用共享生产 handler。每个测试使用单连接内存 SQLite 并运行完整迁移。

```powershell
cd src-tauri
cargo test --lib -- --test-threads=1
```

Windows 上启用 Tauri `test` feature 会链接原生对话框。`build.rs` 为测试程序提供 Common Controls v6 manifest 依赖，避免加载不含 `TaskDialogIndirect` 的 v5 `comctl32.dll`。
