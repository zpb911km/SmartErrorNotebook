<template>
  <div class="conflict-item">
    <!-- 记录预览 -->
    <div class="conflict-item__preview" :class="{ 'conflict-item__preview--selected': selected }">
      <div class="conflict-item__header">
        <span class="conflict-item__type">错题 #{{ id.slice(0, 8) }}</span>
        <span class="conflict-item__versions">
          本地 v{{ localVersion }} vs 服务端 v{{ serverVersion }}
        </span>
      </div>

      <!-- 内容摘要 -->
      <div class="conflict-item__body">
        <div class="conflict-section">
          <h4 class="conflict-section__title">本地版本</h4>
          <p class="conflict-section__content">{{ localPreview }}</p>
        </div>
        <div class="divider"></div>
        <div class="conflict-section">
          <h4 class="conflict-section__title">服务端版本</h4>
          <p class="conflict-section__content">{{ serverPreview }}</p>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="conflict-item__actions" v-if="!resolved">
      <button class="btn btn--local" @click="handleResolve('keep_local')">
        保留本地版本
      </button>
      <button class="btn btn--remote" @click="handleResolve('keep_remote')">
        保留服务端版本
      </button>
    </div>

    <!-- 已解决状态 -->
    <div v-else class="conflict-item__resolved">
      <span class="resolved-badge">
        {{ resolvedChoice === 'keep_local' ? '已保留本地版本' : '已使用服务端版本' }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { ConflictInfo } from '../apis/sync';

interface Props {
  conflict: ConflictInfo;
  resolvedId: Set<string>;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'resolve', id: string, resolution: { id: string; resolution: string; final_deleted_at?: number | null }): void;
}>();

const id = computed(() => props.conflict.id);
const localVersion = computed(() => props.conflict.local_version);
const serverVersion = computed(() => props.conflict.server_version);
const selected = computed(() => false); // 暂时不使用选中状态

const localPreview = computed(() => {
  // TODO: 从本地数据库获取实际内容预览
  return '(查看完整内容请打开编辑)';
});

const serverPreview = computed(() => {
  // TODO: 从服务端数据中获取预览
  return '(来自其他设备)';
});

const resolved = computed(() => props.resolvedId.has(props.conflict.id));
const resolvedChoice = ref<'keep_local' | 'keep_remote' | null>(null);

const handleResolve = (choice: 'keep_local' | 'keep_remote') => {
  const finalDeletedAt =
    choice === 'keep_local'
      ? (props.conflict.local_deleted ? null : Date.now())
      : (props.conflict.server_deleted ? Date.now() : null);

  resolvedChoice.value = choice;
  emit('resolve', props.conflict.id, {
    id: props.conflict.id,
    resolution: choice,
    final_deleted_at: finalDeletedAt,
  });
};
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

/* 预览区域 */
.conflict-item__preview {
  background: #f9fafb;
  border-radius: 8px;
  overflow: hidden;
}

.conflict-item__preview--selected {
  background: #eff6ff;
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
}

.conflict-item__versions {
  font-size: 0.75rem;
  color: #374151;
  font-family: monospace;
}

.conflict-item__body {
  padding: 1rem;
}

.divider {
  width: 2px;
  height: 2rem;
  background: #e5e7eb;
  margin: 0 1rem;
}

/* 冲突段 */
.conflict-section {
  flex: 1;
}

.conflict-section__title {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  margin: 0 0 0.5rem;
  text-transform: uppercase;
}

.conflict-section__content {
  font-size: 0.875rem;
  color: #374151;
  line-height: 1.5;
  margin: 0;
}

/* 操作按钮 */
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

/* 已解决状态 */
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
