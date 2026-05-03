<template>
  <div class="markdown-textarea">
    <textarea
      ref="textareaRef"
      v-bind="$attrs"
      :class="['markdown-textarea__input', textareaClass]"
      :value="modelValue"
      @input="handleInput"
    ></textarea>

    <div v-if="showPreview" :class="['markdown-textarea__preview', previewClass]">
      <div v-if="previewTitle" class="markdown-textarea__preview-title">{{ previewTitle }}</div>
      <div class="markdown-textarea__preview-content markdown-body" v-html="renderedMarkdown"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
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

defineOptions({
  inheritAttrs: false
})

interface Props {
  modelValue?: string
  showPreview?: boolean
  previewTitle?: string
  textareaClass?: string
  previewClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  showPreview: true,
  previewTitle: 'Markdown 预览',
  textareaClass: '',
  previewClass: ''
})

const emit = defineEmits<{
  (event: 'update:modelValue', value: string): void
}>()

const textareaRef = ref<HTMLTextAreaElement | null>(null)

const renderedMarkdown = computed(() => {
  const normalized = (props.modelValue || '')
    .replace(/\\\[/g, '$$')
    .replace(/\\\]/g, '$$')
    .replace(/\\\(/g, '$')
    .replace(/\\\)/g, '$')

  return marked.parse(normalized, {
    breaks: true,
    gfm: true
  }) as string
})

const handleInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}

const focus = () => textareaRef.value?.focus()
const blur = () => textareaRef.value?.blur()
const select = () => textareaRef.value?.select()

defineExpose({
  focus,
  blur,
  select,
  el: textareaRef
})
</script>

<style scoped>
.markdown-textarea {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.markdown-textarea__input {
  width: 100%;
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

.markdown-textarea__preview {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  overflow: hidden;
}

.markdown-textarea__preview-title {
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-color);
}

.markdown-textarea__preview-content {
  padding: 12px;
  line-height: 1.6;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-word;
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
</style>
