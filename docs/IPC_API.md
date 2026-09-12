# 新版 IPC API

新版 Tauri IPC API 是 Smart Error Notebook 面向应用层的契约。命令使用 `snake_case`；命令参数和响应字段使用 `camelCase`。旧版兼容 API 仍记录在[旧版 IPC API 参考手册](API_REFERENCE.md)中，其所有命令均带有 `legacy_` 前缀。

## 约定

- ID 是 UUID 字符串；空字符串无效，且绝不用于表示 `null`。
- 时间是 UTC RFC 3339 字符串。
- 枚举值使用 `SCREAMING_SNAKE_CASE`。
- 可选值使用 `null` 表示。Question 和 Source 更新会替换完整的可编辑状态：
  每个更新字段都必须提供，`null` 清除可空字段，`[]` 清除 Question 关系。
- 同步状态、同步版本和墓碑字段不属于传输契约。删除命令会创建用于同步的
  内部墓碑。
- 除无参数的科目和标签列表外，命令接收一个位于顶层、针对具体操作的
  `request` 对象。

命令参数反序列化完成后，应用错误会序列化为：

```ts
interface IpcError {
  code:
    | 'INVALID_ARGUMENT'
    | 'NOT_FOUND'
    | 'MISSING_REFERENCE'
    | 'RESOURCE_IN_USE'
    | 'CORRUPTED_DATA'
    | 'STORAGE_ERROR'
  message: string
  details?: unknown
}
```

命令处理器运行前产生的错误——例如缺少顶层 `request`、缺少必填更新字段，或字段的传输类型错误——属于 Tauri/Serde 传输错误，并以字符串返回。其措辞不是稳定的应用契约。应用代码应使用 `src/api/` 下的类型化封装，以便在 TypeScript 检查期间拒绝格式错误的请求。

`NOT_FOUND` 的详细信息 `details` 始终包含 `entity`；按 ID 寻址的操作还会将请求的 UUID 放在 `id` 中（没有可用于查询的 ID 时省略该字段）。

## 命令

| 领域 | 命令 |
| --- | --- |
| 科目 | `list_subjects`, `create_subject`, `update_subject`, `delete_subject` |
| 来源 | `get_source`, `list_sources`, `create_source`, `update_source`, `delete_source`, `delete_sources` |
| 标签 | `list_tags`, `create_tag`, `update_tag`, `delete_tag` |
| 附件 | `create_attachment`, `get_attachment`, `delete_attachment` |
| 题目 | `list_questions`, `get_question`, `create_question`, `update_question`, `delete_question` |
| 复习 | `get_library_statistics`, `submit_review`, `reset_review_progress`, `get_srs_data`, `list_srs_data` |

科目写命令接收针对具体操作的 `request` 对象。创建和更新返回 `{ subject }`，删除返回 `{ id }`；列表不接收 request，并返回 `{ subjects }`。科目名称和颜色会按输入原样存储。

来源和标签命令采用相同的、针对具体操作的请求/响应模式。来源的创建、获取和更新返回 `{ source }`，列表返回 `{ sources }`，删除返回 `{ id }`。批量删除接收 `{ ids }` 并返回规范化后的 `{ ids }`。即使值为 `null`，来源更新也要求提供 `subjectId`、`book`、`chapter` 和 `knowledge`，因为更新会替换完整的可编辑状态。删除来源会保留其题目，并在同一事务内将这些题目的 `sourceId` 设为 `null`。删除科目会保留其来源和题目，以原子方式清除这些来源的 `subjectId`，并且仅为科目写入墓碑。标签创建和更新返回 `{ tag }`，列表返回 `{ tags }`，删除返回 `{ id }`。即使另一个未删除的来源具有相同属性，`create_source` 也始终创建新记录。

前端来源解析服务会在单个前端实例内合并属性相同且正在进行的来源解析。完成的结果不会缓存；后续调用会再次查询当前资源。这不会为新版 IPC 强制施加属性唯一性，也不会对跨窗口请求进行去重。

附件创建和读取独立于题目写入。`create_attachment` 校验并存储图像，然后返回 `{ id }`；`get_attachment` 返回包含 MIME 类型、Base64 数据和 SHA-256 的 `{ attachment }`；删除返回 `{ id }`。图像解码后限制为 10 MiB，且声明的MIME 类型必须与检测到的内容一致。

规范的 TypeScript 请求和响应定义按领域组织在 `src/types/` 下，类型化命令封装也按相同的领域划分组织在 `src/api/` 下。两个目录都通过 `index.ts` 汇总导出文件提供稳定的公共导入。业务 UI 使用新版类型和独立的查询、编辑服务，详见[前端架构](FRONTEND_ARCHITECTURE.md)。

## 题目及相关输出

`create_question` 和 `update_question` 是题目的写入边界。它们保存题目字段，并通过 UUID 替换 Tag 和 Attachment 关系。题目创建会在同一个应用事务中初始化SRS。Attachment 生命周期操作保持独立，由客户端显式组合。仓储会在与题目写入相同的事务中校验引用资源。初始 SRS 难度始终使用领域模型的 FSRS 默认值，不由客户端提供。

```ts
const tag = await invoke<CreateTagResponse>('create_tag', {
  request: { name: 'careless', color: '#ff0000' }
})
const attachment = await invoke<CreateAttachmentResponse>('create_attachment', {
  request: { mimeType: 'image/png', base64Data }
})
const created = await invoke<CreateQuestionResponse>('create_question', {
  request: {
    sourceId,
    questionType: 'SHORT_ANSWER',
    stem: 'What is 2 + 2?',
    correctAnswer: '4',
    explanation: null,
    note: null,
    tagIds: [tag.tag.id],
    attachmentIds: [attachment.id]
  }
})
const question = created.question
```

题目写入只接受未删除的 Tag 和 Attachment UUID。按属性复用 Tag 以及创建附件属于客户端编排职责。

题目更新要求提供 `sourceId`、`questionType`、`stem`、`correctAnswer`、`explanation`、`note`、`tagIds` 和 `attachmentIds`。调用方必须发送完整的可编辑状态；省略字段表示请求格式错误，而不是保持该字段不变。

题目的创建、更新、获取、删除和列表命令都接收 `request` 对象。题目不接收 `subjectId`；其科目由来源推导。UI 只选择科目时，来源解析服务会创建或复用仅含科目的来源。未分类题目的 `sourceId` 可以为 `null`。包括 `stem` 在内的题目文本字段会按输入原样存储。

创建、更新和获取返回 `{ question }`；删除返回 `{ id }`。`QuestionData` 包含 `sourceId`、`tagIds` 和 `attachmentIds`，而不是展开后的关联对象。删除命令会保留内部墓碑；删除题目还会为其 SRS 记录写入墓碑并移除关系，而 Attachment和可复用 Tag 保持独立的生命周期。

## 读取与复习

`list_questions` 返回 `{ items, total }`。其可选过滤器支持 `search`、`book`、`chapter`、`knowledge`、`tagIds`（任一匹配）、`updatedSince`（`updatedAt >= updatedSince`）和 `reviewState`。排序支持 `UPDATED_AT_ASC`、`UPDATED_AT_DESC`、`MASTERY_ASC`、`MASTERY_DESC`、`ID_ASC` 和 `ID_DESC`；分页为可选。提供 `sort` 时会按声明顺序应用，例如 `['MASTERY_ASC', 'UPDATED_AT_DESC', 'ID_ASC']`。省略它或提供空数组表示不要求任何顺序保证。调用方只提供主排序时，不隐含任何次级排序。科目过滤由前端查询服务在应用客户端分页前完成。

分页条目和总数在同一个仓储事务中读取。资料库统计同样会在一个事务中读取题目和 SRS 总数，因此每个响应都表示同一个一致的数据库快照。

`get_question` 使用与创建、更新、列表及复习操作相同的结构返回 `{ question }`。客户端独立加载关联资源。

复习队列由 `list_questions` 组合而成。待复习卡片使用 `{ request: { filter: { reviewState: 'DUE' }, sort: ['MASTERY_ASC'], limit } }` 读取；全部卡片读取省略 `reviewState`。响应仍为 `{ items, total }`，其中 `total` 是分页前的匹配数量。

`get_library_statistics` 要求调用方提供 RFC 3339 `at` 时间戳，并返回 `{ statistics }`；`dueCount` 在该时间点求值。`submit_review` 要求提供`reviewedAt`，`reset_review_progress` 要求提供 `resetAt`。两者均为调用方提供的 RFC 3339 时间戳，且两个命令都通过 `{ srs }` 返回 SRS 数据（`submit_review` 还返回 `nextIntervalDays`）。`get_srs_data` 和 `list_srs_data` 要求调用方提供 RFC 3339 `at` 时间戳，并分别以 `{ srs }` 和 `{ items }` 暴露 SRS 读取结果。SRS 记录存在上次复习时间时，`resetAt` 不得早于该时间；违反约束会返回 `INVALID_ARGUMENT` 且不更改数据。创建时间和元数据更新时间不施加额外下限，这与复习行为一致。

## 兼容性

Subject、Source、Tag、Attachment 和 Question 具有独立的生命周期。后续 Question 写入失败时，先前成功创建的资源仍保持提交状态；删除 Question 也不会隐式删除其 Attachment。前端编辑会话显式组合各资源操作并保留已成功资源的 ID；它不是事务或聚合边界，不使用具有级联效果的删除命令模拟回滚。如果 Question 写入已提交，但随后请求的过期 Attachment 删除失败，UI 保留已提交结果，并提供单独的附件清理重试。写入后的数据补全始终是单独的读取操作。

新版和旧版命令会同时注册，并操作同一个规范化数据库。本地清理、同步协议维护和文件打开关联命令在迁移期间有意仅由旧版接口提供。
