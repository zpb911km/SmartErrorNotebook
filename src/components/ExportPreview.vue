<template>
  <div class="export-preview" ref="previewRef">
    <!-- 页头 -->
    <div class="preview-header">
      <h1 class="preview-title">错题集</h1>
      <div class="preview-meta">
        导出日期：{{ exportDate }} &nbsp;|&nbsp; 共 {{ questions.length }} 题
      </div>
    </div>

    <!-- 题目列表 -->
    <div
      v-for="(question, index) in questions"
      :key="question.id || index"
      class="preview-card"
    >
      <div class="preview-number">第 {{ index + 1 }} 题</div>

      <div class="preview-section">
        <div class="preview-label">题目</div>
        <div
          class="preview-content markdown-body"
          v-html="renderContent(question.prompt)"
        ></div>
      </div>

      <div v-if="question.answer" class="preview-section">
        <div class="preview-label answer-label">参考答案</div>
        <div
          class="preview-content markdown-body"
          v-html="renderContent(question.answer)"
        ></div>
      </div>

      <div v-if="question.analysis" class="preview-section">
        <div class="preview-label analysis-label">解析</div>
        <div
          class="preview-content markdown-body"
          v-html="renderContent(question.analysis)"
        ></div>
      </div>
    </div>

    <!-- 页脚 -->
    <div class="preview-footer">
      由 SmartErrorNotebook 生成
    </div>

    <!-- 页码占位 -->
    <div class="preview-page-number"></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ErrorQuestion } from '../types'

interface Props {
  questions: ErrorQuestion[]
}

const props = defineProps<Props>()

const exportDate = computed(() => {
  return new Date().toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
})

/**
 * 安全渲染 Markdown 为 HTML
 * 支持：粗体、斜体、代码块、行内代码
 */
function renderContent(text: string | undefined | null): string {
  if (!text) return ''

  let html = String(text)
    // 转义 HTML 特殊字符
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')

  // 代码块
  html = html.replace(/```(\w*)\n([\s\S]*?)```/g, (_match: string, _lang: string, code: string) => {
    const escaped = code
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
    return `<pre><code>${escaped}</code></pre>`
  })

  // 行内代码
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>')

  // 粗体
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')

  // 斜体
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>')

  // 换行
  html = html.replace(/\n/g, '<br>')

  return html
}
</script>

<style scoped>
.export-preview {
  font-family: "Microsoft YaHei", "PingFang SC", "Noto Sans SC", sans-serif;
  color: #333;
  line-height: 1.6;
  padding: 20px;
  background: #fff;
}

.preview-header {
  text-align: center;
  padding: 20px 0;
  border-bottom: 2px solid #1976d2;
  margin-bottom: 20px;
}

.preview-title {
  font-size: 24px;
  color: #1976d2;
  margin: 0 0 8px;
}

.preview-meta {
  font-size: 14px;
  color: #666;
}

.preview-card {
  padding: 16px;
  margin-bottom: 16px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  page-break-inside: avoid;
}

.preview-number {
  font-size: 18px;
  font-weight: bold;
  color: #1976d2;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #eee;
}

.preview-section {
  margin-bottom: 12px;
}

.preview-label {
  font-size: 14px;
  font-weight: 600;
  color: #555;
  margin-bottom: 4px;
}

.answer-label {
  color: #43a047;
}

.analysis-label {
  color: #e65100;
}

.preview-content {
  font-size: 14px;
  color: #333;
}

.preview-content :deep(pre) {
  background: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
  overflow-x: auto;
  font-size: 13px;
  font-family: "Consolas", "Courier New", monospace;
}

.preview-content :deep(code) {
  background: #f5f5f5;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 13px;
  font-family: "Consolas", "Courier New", monospace;
}

.preview-content :deep(strong) {
  font-weight: 700;
}

.preview-content :deep(em) {
  font-style: italic;
}

.preview-footer {
  text-align: center;
  font-size: 12px;
  color: #999;
  padding: 10px 0;
}

.preview-page-number {
  text-align: center;
  font-size: 12px;
  color: #999;
  padding: 5px;
}
</style>
