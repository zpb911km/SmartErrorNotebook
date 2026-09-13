<template>
  <div
    class="markdown-textarea"
    :class="{ 'is-previewing': viewMode === 'preview' }"
  >
    <q-tabs
      v-if="!props.readonly && showPreview"
      :model-value="viewMode"
      dense
      align="left"
      active-color="primary"
      @update:model-value="toggleViewMode"
    >
      <q-tab name="edit" label="编辑" icon="edit_note" />
      <q-tab name="preview" label="预览" icon="visibility" />
    </q-tabs>

    <div v-if="viewMode === 'edit'" class="markdown-textarea__stage">
      <textarea
        ref="textareaRef"
        v-bind="$attrs"
        :class="['markdown-textarea__input', textareaClass]"
        :value="modelValue"
        :readonly="props.readonly"
        @input="handleInput"
        @keydown="handleKeydown"
      />

      <div
        v-if="showPreview && $q.screen.width >= 1024"
        class="markdown-textarea__preview-pane"
      >
        <div class="markdown-textarea__preview-header">
          <div class="markdown-textarea__preview-title">
            {{ previewTitle }}
          </div>
        </div>
        <div class="markdown-textarea__preview-segment" :class="previewClass">
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
          />
        </div>
      </div>
    </div>

    <div v-else-if="showPreview" class="markdown-textarea__preview-only">
      <div
        v-if="previewTitle"
        class="markdown-textarea__preview-header markdown-textarea__preview-header--single"
      >
        <div class="markdown-textarea__preview-title">
          {{ previewTitle }}
        </div>
      </div>
      <div class="markdown-textarea__preview-segment" :class="previewClass">
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
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'

import { renderMarkdown } from '../utils/markdown'

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
