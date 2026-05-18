<template>
  <div class="markdown-test-page">
    <div class="card">
      <h2>MarkdownTextarea 组件测试</h2>
      <p class="desc">用于验证输入、预览渲染、属性透传与双向绑定是否正常。</p>

      <div class="toolbar">
        <button class="btn" @click="fillSample">填充示例</button>
        <button class="btn" @click="fillComplexSample">填充复杂示例</button>
        <button class="btn" @click="clearAll">清空</button>
        <label class="switch-row">
          <input type="checkbox" v-model="showPreview" />
          <span>显示预览</span>
        </label>
      </div>

      <MarkdownTextarea
        v-model="content"
        :showPreview="showPreview"
        :previewTitle="previewTitle"
        placeholder="在这里输入 Markdown..."
        rows="10"
      />

      <div class="meta">
        <div><strong>当前字符数：</strong>{{ content.length }}</div>
        <div><strong>当前行数：</strong>{{ lineCount }}</div>
      </div>
    </div>

    <div class="card">
      <h3>透传属性测试（只读）</h3>
      <MarkdownTextarea
        v-model="readonlyDemo"
        :showPreview="true"
        previewTitle="只读预览"
        rows="5"
        readonly
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const previewTitle = ref('Markdown 预览')
const showPreview = ref(true)

const content = ref(
  `# 标题示例\n\n这是一段 **Markdown** 文本。\n\n- 列表项 1\n- 列表项 2\n\n> 这是一条引用\n\n\`\`\`ts\nconst hello = 'world'\nconsole.log(hello)\n\`\`\``
)

const readonlyDemo = ref('这个区域用于测试 `readonly` 属性透传是否生效。')

const lineCount = computed(() => {
  if (!content.value) return 0
  return content.value.split('\n').length
})

const fillSample = () => {
  content.value = `# 数学错题笔记\n\n## 题目\n求函数 $f(x)=x^2-2x+1$ 的最小值。\n\n## 结论\n最小值为 **0**，在 $x=1$ 时取得。\n\n## 复盘\n1. 忘记配方法\n2. 没有先观察函数结构\n\n---\n\n> 下次先尝试配方，再求顶点。`
}

const fillComplexSample = () => {
  content.value = `# 🚀 Markdown 综合能力压力测试

> 该样例用于验证：标题层级、引用、嵌套列表、任务列表、表格、行内代码、代码块、链接、图片、分割线、以及数学公式渲染。

---

## 1. 基础排版

这是 **加粗**、*斜体*、~~删除线~~、以及 \`行内代码\`。

### 1.1 嵌套列表

1. 第一层有序
   1. 第二层有序
   2. 第二层有序
      - 第三层无序 A
      - 第三层无序 B
2. 第一层有序结束

- 无序列表 A
  - 子项 A-1
  - 子项 A-2

### 1.2 任务列表（GFM）

- [x] 已完成：补齐公式渲染
- [x] 已完成：组件全局注册
- [ ] 待完成：增加更多主题样式

---

## 2. 表格与链接

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| MarkdownTextarea | ✅ 正常 | 支持实时预览 |
| KaTeX 公式渲染 | ✅ 正常 | 支持行内与块级公式 |
| 代码高亮 | ⚠️ 基础 | 当前仅基础样式 |

- 文档链接：[Marked 官方文档](https://marked.js.org/)
- 公式库：[KaTeX 官方站点](https://katex.org/)

---

## 3. 代码块
\`\`\`ts
type ErrorRecord = {
  id: string
  subject: 'math' | 'physics' | 'english'
  score: number
}

const records: ErrorRecord[] = [
  { id: 'q1', subject: 'math', score: 65 },
  { id: 'q2', subject: 'physics', score: 72 }
]

const weak = records.filter((item) => item.score < 70)
console.log('weak records:', weak)
\`\`\`

\`\`\`python
from dataclasses import dataclass

@dataclass
class ErrorRecord:
    subject: str
    score: int

records = [ErrorRecord("math", 65), ErrorRecord("physics", 72)]
print([r for r in records if r.score < 70])
\`\`\`

\`\`\`bash
pnpm install
pnpm dev
\`\`\`

\`\`\`json
{
  "name": "SmartErrorNotebook",
  "feature": ["markdown", "katex", "preview"]
}
\`\`\`

\`\`\`sql
SELECT subject, COUNT(*) AS wrong_count
FROM error_questions
GROUP BY subject
ORDER BY wrong_count DESC;
\`\`\`

---

## 4. 数学公式

行内公式： $f(x)=x^2-2x+1=(x-1)^2$

行内公式（LaTeX 括号）： \\(\\sqrt{a^2+b^2}\\)

块级公式：

$$
\\min f(x) = \\min (x-1)^2 = 0, \\quad x=1
$$

块级公式（LaTeX 括号）：

\\[
E=mc^2
\\]

再来一个积分：

$$
\\int_0^1 x^2\\,dx = \\left.\\frac{x^3}{3}\\right|_0^1 = \\frac{1}{3}
$$

---

## 5. 图片与引用

![占位图](https://dummyimage.com/640x140/1976d2/ffffff&text=Markdown+Preview+Test)

> “学习不是为了避免错误，而是为了通过错误更快地逼近正确答案。”

---

### 结尾检查清单

- [x] 预览区是否正常渲染
- [x] 数学公式是否正确显示
- [x] 表格与任务列表样式是否可读

<s>删除线测试</s>
<mark>高亮内容测试</mark>`
}

const clearAll = () => {
  content.value = ''
}
</script>

<style scoped>
.markdown-test-page {
  padding: 20px;
  padding-bottom: 80px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

h2,
h3 {
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.desc {
  margin: 0 0 12px 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.toolbar {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: 12px;
}

.btn {
  border: 1px solid var(--border-color);
  background: var(--input-bg);
  color: var(--text-primary);
  border-radius: 8px;
  padding: 8px 12px;
  cursor: pointer;
}

.btn:active {
  transform: scale(0.98);
}

.switch-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-size: 13px;
}

.meta {
  margin-top: 10px;
  color: var(--text-secondary);
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
</style>
