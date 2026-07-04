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

        <!-- Canvas 动画区域 -->
        <div class="sync-card__canvas-wrap">
          <canvas
            ref="canvasRef"
            class="sync-card__canvas"
            :width="canvasSize"
            :height="canvasSize"
          />
          <!-- 进度覆盖层 -->
          <div class="sync-card__progress-overlay">
            <svg class="progress-ring" viewBox="0 0 100 100">
              <circle class="progress-ring__bg" cx="50" cy="50" r="40" />
              <circle
                class="progress-ring__bar"
                cx="50"
                cy="50"
                r="40"
                :stroke-dasharray="circleDashArray"
                :stroke-dashoffset="circleDashOffset"
                :style="{
                  transition: 'stroke-dashoffset 0.3s ease',
                  '--progress-color': progressColor
                }"
              />
            </svg>
            <div class="progress-ring__text">
              {{ Math.round(progressPercent) }}%
            </div>
          </div>
          <!-- 模式名称标签 -->
          <div class="sync-card__mode-label">{{ modeLabel }}</div>
        </div>

        <!-- 详细统计 -->
        <div class="sync-card__stats">
          <div class="stat-item">
            <span class="stat-label">已传输</span>
            <span class="stat-value"
              >{{ details.pulled + details.pushed }}/{{ total }}</span
            >
          </div>
          <div class="stat-item" v-if="details.conflicts_resolved > 0">
            <span class="stat-label">冲突解决</span>
            <span class="stat-value highlight">{{
              details.conflicts_resolved
            }}</span>
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
import { computed, ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { createAnimation, MODE_NAMES } from '../utils/sync-animations'
import type { AnimationMode } from '../utils/sync-animations'
import { isDarkTheme } from '../utils/sync-animations/utils'

interface Props {
  /** 是否显示 */
  modelValue: boolean
  /** 总记录数 */
  total: number
  /** 当前进度 */
  current: number
  /** 已拉取数量 */
  pulled: number
  /** 已推送数量 */
  pushed: number
  /** 已解决冲突数 */
  conflictsResolved: number
  /** 失败数量 */
  failed: number
  /** 当前阶段 */
  phase: string
  /** 是否显示取消按钮 */
  showCancelButton?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showCancelButton: true
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'cancel'): void
}>()

/* ── 基础计算属性 ── */

const isSyncing = computed(() => props.modelValue)
const progressPercent = computed(() => {
  if (props.total === 0) return 0
  return (props.current / props.total) * 100
})
const progressNormalized = computed(() => progressPercent.value / 100)

const circleDashArray = computed(() => 251.3)
const circleDashOffset = computed(() => {
  const circumference = 251.3
  return circumference - (progressPercent.value / 100) * circumference
})

const progressColor = computed(() => {
  if (props.failed > 0) return '#ef4444' // red
  if (props.conflictsResolved > 0) return '#f59e0b' // amber
  return '#10b981' // green
})

const title = computed(() => {
  switch (props.phase) {
    case 'handshake':
      return '握手中...'
    case 'pulling':
      return '正在拉取数据...'
    case 'pushing':
      return '正在上传数据...'
    case 'resolving_conflicts':
      return '处理冲突...'
    case 'updating':
      return '更新中...'
    default:
      return '同步中'
  }
})

const statusText = computed(() => {
  switch (props.phase) {
    case 'handshake':
      return '分析需要同步的数据'
    case 'pulling':
      return `从服务器获取 ${props.pulled} 条记录`
    case 'pushing':
      return `上传到服务器 ${props.pushed} 条记录`
    case 'resolving_conflicts':
      return `发现 ${props.conflictsResolved} 个冲突待解决`
    case 'updating':
      return '更新本地数据库状态'
    default:
      return ''
  }
})

const details = computed(() => ({
  pulled: props.pulled,
  pushed: props.pushed,
  conflicts_resolved: props.conflictsResolved,
  failed: props.failed,
  total: props.total,
  current: props.current
}))

/* ── Canvas 动画系统 ── */

const canvasRef = ref<HTMLCanvasElement | null>(null)
const canvasSize = 180
const modeLabel = ref('')

let animation: AnimationMode | null = null
let animFrameId: number | null = null
let darkThemePrev = false

/** 初始化新动画（每次同步开始/种子变化时调用） */
function initAnimation() {
  destroyAnimation()

  const canvas = canvasRef.value
  if (!canvas) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // 基于时间戳生成种子，确保每次不同
  const seed = Date.now() ^ Math.floor(Math.random() * 1000000)
  animation = createAnimation(seed)
  modeLabel.value = MODE_NAMES[animation.name] || animation.name

  darkThemePrev = isDarkTheme()
  animation.init(ctx, canvasSize, canvasSize, seed)

  // 启动帧循环
  const loop = () => {
    if (!animation || !canvasRef.value) return

    const ctx2 = canvasRef.value.getContext('2d')
    if (!ctx2) return

    // 检查主题变化
    const currentDark = isDarkTheme()
    if (currentDark !== darkThemePrev) {
      darkThemePrev = currentDark
      // 主题变化时重绘背景
    }

    // 更新（传入进度）
    animation.update(progressNormalized.value, currentDark)

    // 绘制
    animation.draw(ctx2)

    animFrameId = requestAnimationFrame(loop)
  }

  animFrameId = requestAnimationFrame(loop)
}

/** 销毁当前动画 */
function destroyAnimation() {
  if (animFrameId !== null) {
    cancelAnimationFrame(animFrameId)
    animFrameId = null
  }
  if (animation) {
    animation.destroy()
    animation = null
  }
  modeLabel.value = ''
}

/* ── 生命周期 ── */

// 监听显示状态
watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal) {
      nextTick(() => initAnimation())
    } else {
      destroyAnimation()
    }
  }
)

// 监听进度变化（用于恢复 canvas 等）
watch(
  () => props.current,
  () => {
    // 动画循环已经在运行，progress 通过 update 传入
  }
)

onMounted(() => {
  if (props.modelValue) {
    nextTick(() => initAnimation())
  }
})

onBeforeUnmount(() => {
  destroyAnimation()
})

const handleCancel = () => {
  emit('update:modelValue', false)
  emit('cancel')
}
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
  gap: 1.25rem;
  padding: 1.5rem 2rem 2rem;
  min-width: 320px;
  max-width: 400px;
  border-radius: 16px;
  background: var(--card-bg);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

body.dark-theme .sync-card {
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
}

.sync-card__header {
  text-align: center;
}

.sync-card__title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.sync-card__status {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin: 0.5rem 0 0;
}

/* ── Canvas 动画区域 ── */

.sync-card__canvas-wrap {
  position: relative;
  width: 180px;
  height: 180px;
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  flex-shrink: 0;
}

.sync-card__canvas {
  display: block;
  width: 100%;
  height: 100%;
}

/* 进度环覆盖层 */
.sync-card__progress-overlay {
  position: absolute;
  bottom: 8px;
  right: 8px;
  width: 56px;
  height: 56px;
  pointer-events: none;
}

.progress-ring {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
  filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.3));
}

.progress-ring__bg {
  stroke: rgba(255, 255, 255, 0.25);
  fill: none;
  stroke-width: 6;
}

.progress-ring__bar {
  stroke: var(--progress-color, #10b981);
  fill: none;
  stroke-width: 6;
  stroke-linecap: round;
}

.progress-ring__text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 0.625rem;
  font-weight: 700;
  color: #ffffff;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
  line-height: 1;
}

/* 暗色主题修正 */
body.dark-theme .progress-ring__bg {
  stroke: rgba(255, 255, 255, 0.15);
}

/* 模式名称标签 */
.sync-card__mode-label {
  position: absolute;
  bottom: 8px;
  left: 8px;
  font-size: 0.625rem;
  color: rgba(255, 255, 255, 0.6);
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  pointer-events: none;
  font-weight: 500;
  letter-spacing: 0.02em;
}

/* ── 统计信息 ── */

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
  background: var(--bg-secondary);
  border-radius: 8px;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.stat-value {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-value.highlight {
  color: #f59e0b;
}

.stat-value.error {
  color: #ef4444;
}

/* ── 按钮样式 ── */

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
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.btn--outline:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

/* ── 响应式 ── */

@media (max-width: 768px) {
  .sync-card {
    min-width: 280px;
    max-width: 340px;
    padding: 1.25rem 1.5rem 1.5rem;
    gap: 1rem;
  }

  .sync-card__canvas-wrap {
    width: 150px;
    height: 150px;
  }

  .sync-card__progress-overlay {
    width: 48px;
    height: 48px;
    bottom: 6px;
    right: 6px;
  }
}
</style>
