<template>
  <div class="markdown-textarea" :class="{ 'is-previewing': viewMode === 'preview' }">
    <div class="markdown-textarea__toolbar">
      <div class="markdown-textarea__hint"></div>
    </div>

    <div v-if="viewMode === 'edit'" class="markdown-textarea__stage">
      <textarea
        ref="textareaRef"
        v-bind="$attrs"
        :class="['markdown-textarea__input', textareaClass]"
        :value="modelValue"
        @input="handleInput"
        @keydown="handleKeydown"
      ></textarea>

      <div v-if="showPreview" class="markdown-textarea__preview-pane">
        <div class="markdown-textarea__preview-header">
          <div class="markdown-textarea__preview-title">{{ previewTitle }}</div>
          <button type="button" class="markdown-textarea__mode-switch" @click="toggleViewMode">
            {{ viewMode === 'edit' ? '专注预览' : '编辑' }}
          </button>
        </div>
        <div ref="previewRef" class="markdown-textarea__preview-body markdown-body" :class="previewClass">
          <div v-if="previewSegments.length === 0" class="markdown-textarea__preview-empty">
            预览会随编辑内容滚动自动对齐
          </div>
          <div
            v-for="segment in previewSegments"
            :key="segment.id"
            class="markdown-textarea__preview-segment"
            :class="{ 'is-active': segment.id === activeSegmentId }"
            v-html="segment.html"
          ></div>
        </div>
      </div>
    </div>

    <div v-else-if="showPreview" class="markdown-textarea__preview-only">
      <div class="markdown-textarea__preview-header markdown-textarea__preview-header--single">
        <div class="markdown-textarea__preview-title">{{ previewTitle }}</div>
        <button type="button" class="markdown-textarea__mode-switch" @click="toggleViewMode">
          编辑
        </button>
      </div>
      <div class="markdown-textarea__preview-body markdown-body" :class="previewClass">
        <div v-if="previewSegments.length === 0" class="markdown-textarea__preview-empty">
          当前没有可预览内容
        </div>
        <div
          v-for="segment in previewSegments"
          :key="segment.id"
          class="markdown-textarea__preview-segment"
          v-html="segment.html"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { marked } from 'marked'
import markedKatex from 'marked-katex-extension'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'
import 'katex/dist/katex.min.css'

marked.use(
  markedKatex({
    throwOnError: false,
    output: 'html'
  })
)

const renderer = new marked.Renderer()
renderer.code = ({ text, lang }) => {
  const language = lang ?? ''
  const highlighted = language && hljs.getLanguage(language)
    ? hljs.highlight(text, { language }).value
    : hljs.highlightAuto(text).value
  const langClass = language ? `language-${language}` : ''
  return `<pre><code class="hljs ${langClass}">${highlighted}</code></pre>`
}
marked.use({ renderer })

defineOptions({ inheritAttrs: false })

interface Props {
  modelValue?: string
  showPreview?: boolean
  previewTitle?: string
  textareaClass?: string
  previewClass?: string
  defaultViewMode?: 'edit' | 'preview'
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  showPreview: true,
  previewTitle: 'Markdown 预览',
  textareaClass: '',
  previewClass: '',
  defaultViewMode: 'preview'
})

const emit = defineEmits<{ (event: 'update:modelValue', value: string): void }>()

const textareaRef = ref<HTMLTextAreaElement | null>(null)
const previewRef = ref<HTMLDivElement | null>(null)
const viewMode = ref<'edit' | 'preview'>(props.defaultViewMode)
const activeSegmentId = ref('segment-0')
watch(() => props.defaultViewMode, (value) => {
  viewMode.value = value
})

const normalizeMarkdown = (value: string) => {
  return (value || '')
    .replace(/\\\[/g, '$$')
    .replace(/\\\]/g, '$$')
    .replace(/\\\(/g, '$')
    .replace(/\\\)/g, '$')
}

const renderMarkdown = (value: string) => {
  const normalized = normalizeMarkdown(value)
  return marked.parse(normalized, { breaks: true, gfm: true }) as string
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

  return [{
    id: 'segment-0',
    html
  }]
})

const handleInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}

const focus = () => textareaRef.value?.focus()
const blur = () => textareaRef.value?.blur()
const select = () => textareaRef.value?.select()

const toggleViewMode = async () => {
  if (!props.showPreview) return

  viewMode.value = viewMode.value === 'edit' ? 'preview' : 'edit'
  await nextTick()
  if (viewMode.value === 'edit') {
    await focusAtStart()
  }
}

const returnToEdit = async () => {
  if (!props.showPreview) return
  viewMode.value = 'edit'
  await nextTick()
  await focusAtStart()
}

const handleKeydown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter' && props.showPreview) {
    event.preventDefault()
    toggleViewMode()
    return
  }

  if (event.key === 'Escape' && props.showPreview) {
    event.preventDefault()
    returnToEdit()
  }
}

watch(() => props.modelValue, () => {
  void 0
})

onMounted(() => {
  void 0
})

onBeforeUnmount(() => {
  void 0
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
  min-height: 360px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  background: var(--input-bg);
  color: var(--text-primary);
  box-sizing: border-box;
  resize: vertical;
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
  min-height: 360px;
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
  padding: 8px 10px;
  border-radius: 8px;
  transition: background-color 0.2s ease;
}

.markdown-textarea__preview-segment.is-active {
  background: rgba(25, 118, 210, 0.08);
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
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
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
</style>
