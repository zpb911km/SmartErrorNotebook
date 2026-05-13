<template>
  <Teleport to="body">
    <div v-show="isSyncing" class="sync-overlay">
      <!-- 背景遮罩 -->
      <div class="sync-overlay__mask"></div>

      <!-- 主卡片 -->
      <div class="sync-card">
        <!-- 标题和状态 -->
        <div class="sync-card__header">
          <h2 class="sync-card__title">{{ title }}</h2>
          <p class="sync-card__status">{{ statusText }}</p>
        </div>

        <!-- 动画图标 -->
        <div class="sync-card__icon" :class="{ 'sync-card__icon--animating': isSyncing }">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
            <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
            <path d="M16 16h5v5" />
          </svg>
        </div>

        <!-- 环形进度条 -->
        <div class="sync-card__progress">
          <svg class="progress-circle" :viewBox="circleViewBox">
            <!-- 背景圆环 -->
            <circle
              class="progress-circle__bg"
              cx="50"
              cy="50"
              r="40"
            />
            <!-- 进度圆环 -->
            <circle
              class="progress-circle__bar"
              cx="50"
              cy="50"
              r="40"
              :stroke-dasharray="circleDashArray"
              :stroke-dashoffset="circleDashOffset"
              :style="{
                transition: 'stroke-dashoffset 0.3s ease',
                '--progress-color': progressColor,
              }"
            />
          </svg>
          <div class="progress-circle__text">
            {{ Math.round(progressPercent) }}%
          </div>
        </div>

        <!-- 详细统计 -->
        <div class="sync-card__stats">
          <div class="stat-item">
            <span class="stat-label">已传输</span>
            <span class="stat-value">{{ details.pulled + details.pushed }}/{{ total }}</span>
          </div>
          <div class="stat-item" v-if="details.conflicts_resolved > 0">
            <span class="stat-label">冲突解决</span>
            <span class="stat-value highlight">{{ details.conflicts_resolved }}</span>
          </div>
          <div class="stat-item" v-if="details.failed > 0">
            <span class="stat-label">失败</span>
            <span class="stat-value error">{{ details.failed }}</span>
          </div>
        </div>

        <!-- 取消按钮 -->
        <button
          v-if="showCancelButton"
          class="btn btn--outline btn--sm"
          @click="handleCancel"
        >
          取消
        </button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  /** 是否显示 */
  modelValue: boolean;
  /** 总记录数 */
  total: number;
  /** 当前进度 */
  current: number;
  /** 已拉取数量 */
  pulled: number;
  /** 已推送数量 */
  pushed: number;
  /** 已解决冲突数 */
  conflictsResolved: number;
  /** 失败数量 */
  failed: number;
  /** 当前阶段 */
  phase: string;
  /** 是否显示取消按钮 */
  showCancelButton?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  showCancelButton: true,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'cancel'): void;
}>();

const isSyncing = computed(() => props.modelValue);
const progressPercent = computed(() => {
  if (props.total === 0) return 0;
  return (props.current / props.total) * 100;
});

const circleViewBox = computed(() => `0 0 100 100`);
const circleDashArray = computed(() => {
  // 圆周长 = 2 * PI * r = 2 * PI * 40 ≈ 251.3
  return 251.3;
});
const circleDashOffset = computed(() => {
  const circumference = 251.3;
  return circumference - (progressPercent.value / 100) * circumference;
});

const progressColor = computed(() => {
  if (props.failed > 0) return '#ef4444'; // red
  if (props.conflictsResolved > 0) return '#f59e0b'; // amber
  return '#10b981'; // green
});

const title = computed(() => {
  switch (props.phase) {
    case 'handshake':
      return '握手中...';
    case 'pulling':
      return '正在拉取数据...';
    case 'pushing':
      return '正在上传数据...';
    case 'resolving_conflicts':
      return '处理冲突...';
    case 'updating':
      return '更新中...';
    default:
      return '同步中';
  }
});

const statusText = computed(() => {
  switch (props.phase) {
    case 'handshake':
      return '分析需要同步的数据';
    case 'pulling':
      return `从服务器获取 ${props.pulled}/${Math.max(props.pulled, props.pushed)} 条记录`;
    case 'pushing':
      return `上传到服务器 ${props.pushed}/${Math.max(props.pulled, props.pushed)} 条记录`;
    case 'resolving_conflicts':
      return `发现 ${props.conflictsResolved} 个冲突待解决`;
    case 'updating':
      return '更新本地数据库状态';
    default:
      return '';
  }
});

const details = computed(() => ({
  pulled: props.pulled,
  pushed: props.pushed,
  conflicts_resolved: props.conflictsResolved,
  failed: props.failed,
  total: props.total,
  current: props.current,
}));

const handleCancel = () => {
  emit('update:modelValue', false);
  emit('cancel');
};
</script>

<style scoped>
.sync-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.sync-overlay__mask {
  position: absolute;
  inset: 0;
}

.sync-card {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  padding: 2rem;
  min-width: 320px;
  max-width: 400px;
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.sync-card__header {
  text-align: center;
}

.sync-card__title {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.sync-card__status {
  font-size: 0.875rem;
  color: #6b7280;
  margin: 0.5rem 0 0;
}

/* 动画图标 */
.sync-card__icon {
  width: 64px;
  height: 64px;
  color: #10b981;
}

.sync-card__icon svg {
  width: 100%;
  height: 100%;
}

.sync-card__icon--animating {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.8;
  }
}

/* 环形进度条 */
.sync-card__progress {
  position: relative;
  width: 120px;
  height: 120px;
}

.progress-circle {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.progress-circle__bg {
  stroke: #e5e7eb;
  fill: none;
  stroke-width: 8;
}

.progress-circle__bar {
  stroke: var(--progress-color);
  fill: none;
  stroke-width: 8;
  stroke-linecap: round;
}

.progress-circle__text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 1.5rem;
  font-weight: 600;
  color: #1f2937;
}

/* 统计信息 */
.sync-card__stats {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  justify-content: center;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.5rem 1rem;
  background: #f9fafb;
  border-radius: 8px;
}

.stat-label {
  font-size: 0.75rem;
  color: #6b7280;
  margin-bottom: 0.25rem;
}

.stat-value {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
}

.stat-value.highlight {
  color: #f59e0b;
}

.stat-value.error {
  color: #ef4444;
}

/* 按钮样式 */
.btn {
  border: none;
  border-radius: 8px;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn--sm {
  padding: 0.375rem 0.75rem;
}

.btn--outline {
  background: transparent;
  border: 1px solid #d1d5db;
  color: #6b7280;
}

.btn--outline:hover {
  background: #f9fafb;
  color: #1f2937;
}
</style>
