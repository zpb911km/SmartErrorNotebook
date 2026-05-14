<template>
  <Teleport to="body">
    <div v-show="show" class="conflict-overlay">
      <div class="conflict-overlay__mask" @click.self="handleBackgroundClick"></div>

      <div class="conflict-card">
        <div class="conflict-card__header">
          <h2 class="conflict-card__title">发现 {{ conflicts.length }} 个需要同步的冲突</h2>
          <p class="conflict-card__desc">
            以下记录在多个设备上有不同的修改，请选择要保留的版本
          </p>
        </div>

        <div class="conflict-list">
          <ConflictItem
            v-for="conflict in conflicts"
            :key="conflict.id"
            :conflict="conflict"
            :resolved-choice="resolvedMap.get(conflict.id)?.resolution ?? null"
            @resolve="handleResolve"
          />
        </div>

        <div class="conflict-card__footer">
          <button class="btn btn--outline" @click="handleSkip">
            跳过 (保持本地)
          </button>
          <button
            v-if="resolvedMap.size > 0"
            class="btn btn--confirm"
            @click="confirmResolved"
          >
            确认已选 ({{ resolvedMap.size }})
          </button>
          <button class="btn btn--primary" @click="handleAllRemote">
            全部使用服务端版本
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { ConflictInfo, ResolvedConflict } from '../apis/sync';
import ConflictItem from './ConflictItem.vue';

interface Props {
  modelValue: boolean;
  conflicts: ConflictInfo[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'resolve', resolutions: ResolvedConflict[]): void;
}>();

const resolvedMap = ref<Map<string, ResolvedConflict>>(new Map());

const show = computed(() => props.modelValue && props.conflicts.length > 0);

const handleResolve = (conflictId: string, resolution: ResolvedConflict) => {
  resolvedMap.value.set(conflictId, resolution);
  // force reactivity
  resolvedMap.value = new Map(resolvedMap.value);
};

const confirmResolved = () => {
  if (resolvedMap.value.size > 0) {
    emit('resolve', Array.from(resolvedMap.value.values()));
  }
  emit('update:modelValue', false);
};

const handleSkip = () => {
  const all: ResolvedConflict[] = [
    // already resolved items
    ...Array.from(resolvedMap.value.values()),
    // unresolved ones → keep_local
    ...props.conflicts
      .filter((c) => !resolvedMap.value.has(c.id))
      .map((c) => ({
        id: c.id,
        resolution: 'keep_local' as const,
      })),
  ];
  if (all.length > 0) emit('resolve', all);
  emit('update:modelValue', false);
};

const handleAllRemote = () => {
  const all: ResolvedConflict[] = [
    ...Array.from(resolvedMap.value.values()),
    ...props.conflicts
      .filter((c) => !resolvedMap.value.has(c.id))
      .map((c) => ({
        id: c.id,
        resolution: 'keep_remote' as const,
      })),
  ];
  if (all.length > 0) emit('resolve', all);
  emit('update:modelValue', false);
};

const handleBackgroundClick = () => {
  if (resolvedMap.value.size === props.conflicts.length) {
    confirmResolved();
  }
};
</script>

<style scoped>
.conflict-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.conflict-overlay__mask {
  position: absolute;
  inset: 0;
}

.conflict-card {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  width: 90vw;
  max-width: 600px;
  max-height: 80vh;
  overflow: hidden;
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.conflict-card__header {
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
}

.conflict-card__title {
  font-size: 1.125rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 0.5rem;
}

.conflict-card__desc {
  font-size: 0.875rem;
  color: #6b7280;
  margin: 0;
}

.conflict-list {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.conflict-card__footer {
  display: flex;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
}

.btn {
  border: none;
  border-radius: 8px;
  padding: 0.625rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn--outline {
  flex: 1;
  background: transparent;
  border: 1px solid #d1d5db;
  color: #6b7280;
}

.btn--outline:hover {
  background: #ffffff;
  border-color: #9ca3af;
  color: #1f2937;
}

.btn--primary {
  flex: 1;
  background: #3b82f6;
  color: #ffffff;
}

.btn--primary:hover {
  background: #2563eb;
}

.btn--confirm {
  background: #10b981;
  color: #ffffff;
}

.btn--confirm:hover {
  background: #059669;
}
</style>
