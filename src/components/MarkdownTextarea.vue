<template>
  <div
    class="markdown-textarea"
    :class="{ 'is-previewing': viewMode === 'preview' }"
  >
    <div class="markdown-textarea__toolbar">
      <div class="markdown-textarea__hint"></div>
    </div>

    <div v-if="viewMode === 'edit'" class="markdown-textarea__stage">
      <textarea
        ref="textareaRef"
        v-bind="$attrs"
        :class="['markdown-textarea__input', textareaClass]"
        :value="modelValue"
        :readonly="props.readonly"
        @input="handleInput"
        @keydown="handleKeydown"
      ></textarea>

      <div v-if="showPreview" class="markdown-textarea__preview-pane">
        <div class="markdown-textarea__preview-header">
          <div class="markdown-textarea__preview-title">{{ previewTitle }}</div>
          <button
            v-if="!props.readonly"
            type="button"
            class="markdown-textarea__mode-switch"
            @click="toggleViewMode"
          >
            {{ viewMode === 'edit' ? '专注预览' : '编辑' }}
          </button>
        </div>
        <div
          class="markdown-textarea__preview-segment"
          :class="previewClass"
        >
          <div
            v-if="previewSegments.length === 0"
            class="markdown-textarea__preview-empty"
          >
            当前没有可预览内容
          </div>
          <div
            v-for="segment in previewSegments"
            :key="segment.id"
            class="markdown-textarea__preview-body markdown-body"
            :class="{ 'is-active': segment.id === activeSegmentId }"
            v-html="segment.html"
          ></div>
        </div>
      </div>
    </div>

    <div v-else-if="showPreview" class="markdown-textarea__preview-only">
      <div
        class="markdown-textarea__preview-header markdown-textarea__preview-header--single"
      >
        <div class="markdown-textarea__preview-title">{{ previewTitle }}</div>
        <button
          v-if="!props.readonly"
          type="button"
          class="markdown-textarea__mode-switch"
          @click="toggleViewMode"
        >
          编辑
        </button>
      </div>
      <div
        class="markdown-textarea__preview-segment"
        :class="previewClass"
      >
        <div
          v-if="previewSegments.length === 0"
          class="markdown-textarea__preview-empty"
        >
          当前没有可预览内容
        </div>
        <div
          v-for="segment in previewSegments"
          :key="segment.id"
          class="markdown-textarea__preview-body markdown-body"
          v-html="segment.html"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { Marked } from 'marked'
import markedKatex from 'marked-katex-extension'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import bash from 'highlight.js/lib/languages/bash'
import json from 'highlight.js/lib/languages/json'
import css from 'highlight.js/lib/languages/css'
import sql from 'highlight.js/lib/languages/sql'
import java from 'highlight.js/lib/languages/java'
import cpp from 'highlight.js/lib/languages/cpp'
import rust from 'highlight.js/lib/languages/rust'
import xml from 'highlight.js/lib/languages/xml'
import yaml from 'highlight.js/lib/languages/yaml'
import markdown from 'highlight.js/lib/languages/markdown'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('json', json)
hljs.registerLanguage('css', css)
hljs.registerLanguage('sql', sql)
hljs.registerLanguage('java', java)
hljs.registerLanguage('cpp', cpp)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('markdown', markdown)
// plaintext is built-in, no registration needed
import 'highlight.js/styles/github-dark.css'
import 'katex/dist/katex.min.css'

// 防止 ```markdown 嵌套递归渲染的深度计数器
let _renderDepth = 0

// 创建独立的 marked 实例，避免污染全局 marked
const _marked = new Marked(
  markedKatex({
    throwOnError: false,
    output: 'html',
    nonStandard: true
  }),
  {
    renderer: {
      code({ text, lang }) {
        // AI 经常用 ```markdown ... ``` 包裹返回内容，此时应渲染为 markdown 而非代码高亮
        if (lang?.toLowerCase() === 'markdown' && _renderDepth < 3) {
          _renderDepth++
          try {
            return _marked.parse(text, { breaks: true, gfm: true }) as string
          } finally {
            _renderDepth--
          }
        }
        const language = lang ?? ''
        const highlighted = language && hljs.getLanguage(language)
          ? hljs.highlight(text, { language }).value
          : text  // 无语言标签时不染色，直接展示原文，避免 highlightAuto 猜错
        const langClass = language ? `language-${language}` : ''
        return `<pre><code class="hljs ${langClass}">${highlighted}</code></pre>`
      }
    }
  }
)

defineOptions({ inheritAttrs: false })

interface Props {
  modelValue?: string
  showPreview?: boolean
  previewTitle?: string
  textareaClass?: string
  previewClass?: string
  defaultViewMode?: 'edit' | 'preview'
  readonly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  showPreview: true,
  previewTitle: 'Markdown 预览',
  textareaClass: '',
  previewClass: '',
  defaultViewMode: 'preview',
  readonly: false
})

const emit = defineEmits<{
  (event: 'update:modelValue', value: string): void
}>()

const textareaRef = ref<HTMLTextAreaElement | null>(null)
const viewMode = ref<'edit' | 'preview'>(props.defaultViewMode)
const activeSegmentId = ref('segment-0')
watch(
  () => props.defaultViewMode,
  (value) => {
    viewMode.value = value
  }
)

const normalizeMarkdown = (value: string) => {
  return (value || '')
    .replace(/\\\[/g, '$$')
    .replace(/\\\]/g, '$$')
    .replace(/\\\(/g, '$')
    .replace(/\\\)/g, '$')
}

const renderMarkdown = (value: string) => {
  const normalized = normalizeMarkdown(value)
  // AI 生成的 markdown 常用缩进做视觉对齐，但 marked GFM 会把
  // ≥4空格的缩进行整行误判为 <pre><code> 代码块，导致 KaTeX 无法处理其中的公式。
  // 解决方案：只去掉前面是空行时的 4 空格缩进（此时 marked 才会解析为代码块），
  // 而跟在列表项后面的缩进（列表嵌套）则保留不动。
  let inFence = false
  let prevLineBlank = true  // 文档开头视作"前面是空行"
  const deindented = normalized.split('\n').map(line => {
    const trimmed = line.trim()
    if (/^```/.test(trimmed)) {
      inFence = !inFence
      prevLineBlank = false
      return line
    }
    if (!inFence && prevLineBlank && /^[ ]{4,}(.+)$/.test(line)) {
      prevLineBlank = false
      return line.replace(/^[ ]{4,}/, '')
    }
    prevLineBlank = trimmed === ''
    return line
  }).join('\n')
  return _marked.parse(deindented, { breaks: true, gfm: true }) as string
}

const autoResize = () => {
  const el = textareaRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = el.scrollHeight + 'px'
}

const focusAtStart = async () => {
  await nextTick()
  const el = textareaRef.value
  if (!el) return
  el.setSelectionRange(0, 0)
  el.focus()
}

const previewSegments = computed(() => {
  const html = renderMarkdown(props.modelValue || '')
  if (!html) return []

  return [
    {
      id: 'segment-0',
      html
    }
  ]
})

const handleInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
  autoResize()
}

const focus = () => textareaRef.value?.focus()
const blur = () => textareaRef.value?.blur()
const select = () => textareaRef.value?.select()

const toggleViewMode = async () => {
  if (!props.showPreview || props.readonly) return

  viewMode.value = viewMode.value === 'edit' ? 'preview' : 'edit'
  await nextTick()
  if (viewMode.value === 'edit') {
    autoResize()
    await focusAtStart()
  }
}

const returnToEdit = async () => {
  if (!props.showPreview || props.readonly) return
  viewMode.value = 'edit'
  await nextTick()
  autoResize()
  await focusAtStart()
}

const handleKeydown = (event: KeyboardEvent) => {
  if (
    (event.ctrlKey || event.metaKey) &&
    event.key === 'Enter' &&
    props.showPreview &&
    !props.readonly
  ) {
    event.preventDefault()
    toggleViewMode()
    return
  }

  if (event.key === 'Escape' && props.showPreview && !props.readonly) {
    event.preventDefault()
    returnToEdit()
  }
}

watch(
  () => props.modelValue,
  () => {
    autoResize()
  }
)

onMounted(() => {
  autoResize()
})

defineExpose({ focus, blur, select, el: textareaRef })
</script>

<style scoped>
.markdown-textarea {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.markdown-textarea__toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.markdown-textarea__hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.markdown-textarea__mode-switch {
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  background: var(--card-bg);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
}

.markdown-textarea__mode-switch:hover {
  color: var(--primary-color);
  border-color: var(--primary-color);
}

.markdown-textarea__stage {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(280px, 0.9fr);
  gap: 12px;
  align-items: stretch;
}

.markdown-textarea__input {
  width: 100%;
  /* min-height: 360px; */
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  background: var(--input-bg);
  color: var(--text-primary);
  box-sizing: border-box;
  resize: vertical;
}

.markdown-textarea__input:read-only {
  background: var(--bg-secondary);
  cursor: not-allowed;
  color: var(--text-primary);
  resize: none;
}

.markdown-textarea__input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.markdown-textarea__preview-pane,
.markdown-textarea__preview-only {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.markdown-textarea__preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-color);
}

.markdown-textarea__preview-header--single {
  justify-content: space-between;
}

.markdown-textarea__preview-title {
  font-size: 12px;
  color: var(--text-secondary);
}

.markdown-textarea__preview-header--single {
  border-bottom: 1px solid var(--border-color);
}

.markdown-textarea__preview-body {
  padding: 12px;
  overflow: auto;
  line-height: 1.6;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-word;
}

.markdown-textarea__preview-empty {
  padding: 24px 12px;
  color: var(--text-secondary);
  font-size: 13px;
}

.markdown-textarea__preview-segment {
  background: var(--bg-markdown-primary);
  padding: 8px 10px;
  border-radius: 8px;
  color: var(--text-primary);
  transition: background-color 0.2s ease;
}

.markdown-textarea__preview-segment.is-active {
  background: var(--bg-markdown-primary);
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4),
.markdown-body :deep(h5),
.markdown-body :deep(h6) {
  margin: 0.8em 0 0.4em;
  font-weight: 600;
}

.markdown-body :deep(p) {
  margin: 0.5em 0;
}

.markdown-body :deep(code) {
  background: rgba(25, 118, 210, 0.12);
  padding: 2px 6px;
  font-family:
    ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono',
    'Courier New', monospace;
}

.markdown-body :deep(pre) {
  background: #0f172a;
  padding: 10px;
  overflow-x: auto;
}

.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  color: #e2e8f0;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 20px;
  margin: 0.5em 0;
}

.markdown-body :deep(blockquote) {
  margin: 0.5em 0;
  padding-left: 10px;
  border-left: 3px solid var(--border-color);
  color: var(--text-secondary);
}

.markdown-body :deep(a) {
  color: var(--primary-color);
  text-decoration: underline;
}

.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid var(--border-color);
  padding: 6px 8px;
}

@media (max-width: 960px) {
  .markdown-textarea__stage {
    grid-template-columns: 1fr;
  }
}

.markdown-textarea__preview-only {
  margin-top: 12px;
}

/* 只预览模式下的预览区域样式 */
.markdown-textarea__preview-only .markdown-textarea__preview-body {
  max-height: 70vh;
  overflow-y: auto;
  color: var(--text-primary);
}

</style>
