<template>
  <div class="conflict-item">
    <div class="conflict-item__preview">
      <div class="conflict-item__header">
        <span class="conflict-item__type">{{ tableName || '记录' }}</span>
        <span class="conflict-item__versions">
          本地 v{{ localVersion }} vs 服务端 v{{ serverVersion }}
        </span>
      </div>

      <div class="conflict-item__body">
        <div class="conflict-section">
          <h4 class="conflict-section__title">本地版本</h4>
          <pre class="conflict-section__content json-preview">{{
            localPreviewText
          }}</pre>
        </div>
        <div class="divider"></div>
        <div class="conflict-section">
          <h4 class="conflict-section__title">服务端版本</h4>
          <pre class="conflict-section__content json-preview">{{
            serverPreviewText
          }}</pre>
        </div>
      </div>
    </div>

    <div class="conflict-item__actions" v-if="!resolved">
      <button class="btn btn--local" @click="handleResolve('keep_local')">
        保留本地版本
      </button>
      <button class="btn btn--remote" @click="handleResolve('keep_remote')">
        保留服务端版本
      </button>
    </div>

    <div v-else class="conflict-item__resolved">
      <span class="resolved-badge">
        {{
          currentChoice === 'keep_local'
            ? '已选择: 本地版本'
            : '已选择: 服务端版本'
        }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { downloadRecord } from '../apis/sync'
import type { ConflictInfo, ResolvedConflict } from '../apis/sync'

const PREVIEW_MAX_LENGTH = 50

interface Props {
  conflict: ConflictInfo
  resolvedChoice: 'keep_local' | 'keep_remote' | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'resolve', id: string, resolution: ResolvedConflict): void
}>()

const localVersion = computed(() => props.conflict.local_version)
const serverVersion = computed(() => props.conflict.server_version)
const resolved = computed(() => props.resolvedChoice !== null)
const currentChoice = computed(() => props.resolvedChoice)

// ---------- 数据加载 ----------
const tableName = ref('')
const localRaw = ref<any>(null)
const serverRaw = ref<any>(null)

onMounted(async () => {
  // 加载本地数据
  try {
    const local = await invoke<any>('get_record_for_upload', {
      recordId: props.conflict.id
    })
    localRaw.value = local.data
    tableName.value = local.table_name
  } catch {
    /* ignore */
  }

  // 加载服务端数据
  const authKey = localStorage.getItem('auth_key') || ''
  if (authKey) {
    try {
      const server = await downloadRecord(props.conflict.id, authKey)
      serverRaw.value = server.data
      if (!tableName.value) tableName.value = server.table_name
    } catch {
      /* ignore */
    }
  }
})

// ---------- 格式化展示 ----------
function formatPreview(obj: any): string {
  if (!obj) return '(无数据)'
  const cleaned = { ...obj }
  // 附件中的 base64_data 不展示
  if (tableName.value === 'attachments' && 'base64_data' in cleaned) {
    cleaned.base64_data = '[base64 omitted]'
  }
  return Object.entries(cleaned)
    .map(([key, val]) => {
      const str = typeof val === 'object' ? JSON.stringify(val) : String(val)
      const truncated =
        str.length > PREVIEW_MAX_LENGTH
          ? str.slice(0, PREVIEW_MAX_LENGTH) + '...'
          : str
      return `${key}: ${truncated}`
    })
    .join('\n')
}

const localPreviewText = computed(() => formatPreview(localRaw.value))
const serverPreviewText = computed(() => formatPreview(serverRaw.value))

// ---------- 选择 ----------
const handleResolve = (choice: 'keep_local' | 'keep_remote') => {
  emit('resolve', props.conflict.id, {
    id: props.conflict.id,
    resolution: choice
  })
}
</script>

<style scoped>
.conflict-item {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  background: #ffffff;
  border-radius: 12px;
  border: 2px solid #e5e7eb;
  transition: border-color 0.2s;
}

.conflict-item:hover {
  border-color: #d1d5db;
}

.conflict-item__preview {
  background: #f9fafb;
  border-radius: 8px;
  overflow: hidden;
}

.conflict-item__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: #f3f4f6;
  border-bottom: 1px solid #e5e7eb;
}

.conflict-item__type {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  font-family: monospace;
}

.conflict-item__versions {
  font-size: 0.75rem;
  color: #374151;
  font-family: monospace;
}

.conflict-item__body {
  display: flex;
  padding: 1rem;
  gap: 1rem;
  min-height: 60px;
}

.divider {
  width: 2px;
  background: #e5e7eb;
  flex-shrink: 0;
}

.conflict-section {
  flex: 1;
  min-width: 0;
}

.conflict-section__title {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  margin: 0 0 0.5rem;
  text-transform: uppercase;
}

.json-preview {
  font-size: 0.75rem;
  font-family: monospace;
  line-height: 1.5;
  color: #374151;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}

.conflict-item__actions {
  display: flex;
  gap: 0.75rem;
}

.btn {
  flex: 1;
  padding: 0.625rem 1rem;
  border: none;
  border-radius: 8px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn--local {
  background: #10b981;
  color: #ffffff;
}

.btn--local:hover {
  background: #059669;
}

.btn--remote {
  background: #3b82f6;
  color: #ffffff;
}

.btn--remote:hover {
  background: #2563eb;
}

.conflict-item__resolved {
  display: flex;
  justify-content: center;
  padding: 0.75rem;
}

.resolved-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  background: #d1fae5;
  color: #065f46;
  border-radius: 20px;
  font-size: 0.875rem;
  font-weight: 500;
}
</style>
