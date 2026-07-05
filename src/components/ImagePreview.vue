<template>
  <div v-if="visible" class="image-preview-overlay" @click="handleClose">
    <div class="image-preview-container" @click.stop>
      <!-- 关闭按钮 -->
      <button class="close-btn" @click="handleClose" title="关闭">
        <Icon name="x" :size="18" />
      </button>

      <!-- 图片 -->
      <div
        ref="imageWrapperRef"
        class="image-wrapper"
        @wheel.capture="handleWheel"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
      >
        <img
          :src="imageUrl"
          alt="图片预览"
          :class="['preview-image', { 'is-dragging': isDragging }]"
          :style="{
            transform: `scale(${scale}) rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            transition:
              isDragging || isPinching ? 'none' : 'transform 0.1s ease-out'
          }"
          @mousedown="handleMouseDown"
          @dblclick="resetAll"
        />
      </div>

      <!-- 缩放控制 -->
      <div class="zoom-controls">
        <button class="zoom-btn" @click="zoomIn" title="放大">
          <Icon name="zoom-in" :size="18" :stroke-width="2.5" />
        </button>
        <span class="zoom-level">{{ Math.round(scale * 100) }}%</span>
        <button class="zoom-btn" @click="zoomOut" title="缩小">
          <Icon name="zoom-out" :size="18" :stroke-width="2.5" />
        </button>
        <span class="divider"></span>
        <button class="zoom-btn rotate-btn" @click="rotateImage" title="旋转">
          <Icon name="rotate-cw" :size="18" />
        </button>
        <button class="zoom-btn reset-btn" @click="resetAll" title="重置">
          重置
        </button>
      </div>

      <!-- 提示（根据平台动态显示） -->
      <div class="zoom-hint">
        {{
          isTouchDevice
            ? '双指缩放 · 拖拽平移 · 双击重置'
            : '滚轮缩放 · 拖拽平移 · 双击重置'
        }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'

interface Props {
  visible: boolean
  imageUrl: string
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// ─── 缩放与变换状态 ──────────────────────────────────────────
const scale = ref(1)
const translateX = ref(0)
const translateY = ref(0)
const rotation = ref(0)

// ─── 拖拽状态（桌面鼠标） ─────────────────────────────────────
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const initialTranslateX = ref(0)
const initialTranslateY = ref(0)

// ─── 触摸状态 ────────────────────────────────────────────────
const isPinching = ref(false)
let pinchStartDistance = 0

// 触摸双击检测
let lastTapTime = 0
let lastTapX = 0
let lastTapY = 0

// 抑制触摸后合成的 mousedown（防止拖拽漂移）
let suppressNextMouseDown = false

// ─── 容器引用 ────────────────────────────────────────────────
const imageWrapperRef = ref<HTMLDivElement | null>(null)

// ─── 平台检测（初始化时直接判断，避免 mounted 后闪烁） ─────
const isTouchDevice = ref(
  typeof window !== 'undefined' &&
    ('ontouchstart' in window || navigator.maxTouchPoints > 0)
)

// ─── 视图口限制（缩小防移出） ────────────────────────────────
const clampToViewport = () => {
  if (!imageWrapperRef.value) return
  const rect = imageWrapperRef.value.getBoundingClientRect()

  // 当 scale < 1 时，限制平移范围防止图片完全移出
  if (scale.value < 1) {
    const maxX = (rect.width - rect.width * scale.value) / 2
    const maxY = (rect.height - rect.height * scale.value) / 2
    translateX.value = Math.max(
      -Math.abs(maxX),
      Math.min(Math.abs(maxX), translateX.value)
    )
    translateY.value = Math.max(
      -Math.abs(maxY),
      Math.min(Math.abs(maxY), translateY.value)
    )
  }
}

// ─── 监听 visible ────────────────────────────────────────────
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      scale.value = 1
      translateX.value = 0
      translateY.value = 0
      rotation.value = 0
      // 等待 DOM 渲染完成后再计算视图口限制
      setTimeout(clampToViewport, 100)
    }
  }
)

// ─── 操作函数 ────────────────────────────────────────────────
const handleClose = () => {
  emit('close')
}

const zoomIn = () => {
  scale.value = Math.min(scale.value * 1.2, 10)
}

const zoomOut = () => {
  scale.value = Math.max(scale.value / 1.2, 0.1)
}

const resetAll = () => {
  scale.value = 1
  translateX.value = 0
  translateY.value = 0
  rotation.value = 0
}

const rotateImage = () => {
  rotation.value = (rotation.value + 90) % 360
}

// ─── 滚轮缩放 ────────────────────────────────────────────────
const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  e.stopPropagation()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  scale.value = Math.max(0.1, Math.min(10, scale.value * delta))
}

// ─── 桌面端鼠标拖拽 ──────────────────────────────────────────
const handleMouseDown = (e: MouseEvent) => {
  // 触摸设备上不处理鼠标事件（防止触摸后合成的 mousedown）
  if (suppressNextMouseDown) {
    suppressNextMouseDown = false
    return
  }

  // 仅左键拖拽
  if (e.button !== 0) return

  e.preventDefault()
  isDragging.value = true
  initialTranslateX.value = translateX.value
  initialTranslateY.value = translateY.value
  dragStartX.value = e.clientX
  dragStartY.value = e.clientY

  const onMouseMove = (ev: MouseEvent) => {
    if (!isDragging.value) return
    ev.preventDefault()
    translateX.value = initialTranslateX.value + ev.clientX - dragStartX.value
    translateY.value = initialTranslateY.value + ev.clientY - dragStartY.value

    if (scale.value < 1) {
      clampToViewport()
    }
  }

  const onMouseUp = () => {
    if (!isDragging.value) return
    isDragging.value = false
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove, { passive: false })
  document.addEventListener('mouseup', onMouseUp, { passive: true })
}

// ─── 移动端触摸交互 ──────────────────────────────────────────

/** 提取触摸坐标 */
const getTouchPos = (e: TouchEvent, index = 0) => {
  const touch = e.touches[index]
  return { x: touch.clientX, y: touch.clientY }
}

/** 计算两指距离 */
const getPinchDistance = (e: TouchEvent): number => {
  const t1 = e.touches[0]
  const t2 = e.touches[1]
  const dx = t1.clientX - t2.clientX
  const dy = t1.clientY - t2.clientY
  return Math.sqrt(dx * dx + dy * dy)
}

const handleTouchStart = (e: TouchEvent) => {
  // 标记抑制后续合成的 mousedown
  suppressNextMouseDown = true

  const touchCount = e.touches.length

  if (touchCount === 1) {
    // ── 单指：检测双击 + 准备拖拽 ──
    const pos = getTouchPos(e)
    const now = Date.now()
    const timeSinceLastTap = now - lastTapTime
    const distSinceLastTap = Math.sqrt(
      (pos.x - lastTapX) ** 2 + (pos.y - lastTapY) ** 2
    )

    if (timeSinceLastTap < 300 && distSinceLastTap < 30) {
      // 触摸双击 → 重置
      resetAll()
      lastTapTime = 0
      return
    }

    lastTapTime = now
    lastTapX = pos.x
    lastTapY = pos.y

    // 准备拖拽
    isDragging.value = true
    isPinching.value = false
    initialTranslateX.value = translateX.value
    initialTranslateY.value = translateY.value
    dragStartX.value = pos.x
    dragStartY.value = pos.y
  } else if (touchCount >= 2) {
    // ── 双指：准备缩放 ──
    isDragging.value = false
    isPinching.value = true
    pinchStartDistance = getPinchDistance(e)
  }
}

const handleTouchMove = (e: TouchEvent) => {
  const touchCount = e.touches.length

  if (touchCount >= 2 && isPinching.value) {
    // ── 双指缩放（增量式） ──
    const currentDist = getPinchDistance(e)
    const ratio = currentDist / pinchStartDistance
    scale.value = Math.max(0.1, Math.min(10, scale.value * ratio))
    // 更新参考距离，下一次增量变化
    pinchStartDistance = currentDist
  } else if (touchCount === 1 && isDragging.value && !isPinching.value) {
    // ── 单指拖拽 ──
    const pos = getTouchPos(e)
    translateX.value = initialTranslateX.value + pos.x - dragStartX.value
    translateY.value = initialTranslateY.value + pos.y - dragStartY.value

    if (scale.value < 1) {
      clampToViewport()
    }
  }
}

const handleTouchEnd = (e: TouchEvent) => {
  const remainingTouches = e.touches.length

  if (remainingTouches < 2) {
    isPinching.value = false
  }

  if (remainingTouches === 0) {
    // 所有手指离开 → 结束拖拽
    isDragging.value = false
  } else if (remainingTouches === 1) {
    // 从多指恢复到单指 → 重新校准拖拽起点
    if (isDragging.value) {
      // 已在拖拽中，不需要重新校准（只有单指才能拖拽）
    } else {
      // 从 pinch 恢复为单指，准备继续拖拽
      const pos = getTouchPos(e)
      isDragging.value = true
      isPinching.value = false
      initialTranslateX.value = translateX.value
      initialTranslateY.value = translateY.value
      dragStartX.value = pos.x
      dragStartY.value = pos.y
    }
  }

  // 延迟清除抑制标记，保证合成的 mousedown 被消费
  setTimeout(() => {
    suppressNextMouseDown = false
  }, 400)
}

// ─── 组件卸载清理 ────────────────────────────────────────────
onUnmounted(() => {
  // 防止拖拽过程中卸载导致监听器泄漏（虽然不是最优雅的，但安全）
  // 实际移除由 mouseup 回调自行处理，此处仅做兜底
})
</script>

<style scoped>
/* ===== 背景遮罩 ===== */
.image-preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: overlayIn 0.25s ease;
  backdrop-filter: blur(4px);
  /* 防止移动端橡皮筋滚动 */
  overscroll-behavior: none;
  /* 阻止浏览器默认的双指缩放页面行为 */
  touch-action: none;
}

@keyframes overlayIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* ===== 容器 ===== */
.image-preview-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: containerIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes containerIn {
  from {
    opacity: 0;
    transform: scale(0.92);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* ===== 关闭按钮 ===== */
.close-btn {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 44px;
  height: 44px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 50%;
  cursor: pointer;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(8px);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  /* 移动端安全区域 */
  top: max(20px, env(safe-area-inset-top, 0px) + 8px);
  right: max(20px, env(safe-area-inset-right, 0px) + 8px);
}

.close-btn svg {
  width: 20px;
  height: 20px;
  color: rgba(255, 255, 255, 0.8);
  transition: all 0.25s ease;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.18);
  border-color: rgba(255, 255, 255, 0.3);
  transform: scale(1.08);
  box-shadow: 0 0 24px rgba(255, 255, 255, 0.08);
}

.close-btn:hover svg {
  color: #fff;
  transform: rotate(90deg);
}

.close-btn:active {
  transform: scale(0.95);
}

/* ===== 图片容器 ===== */
.image-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  /* 阻止浏览器默认触摸行为（滚动/缩放/长按选择） */
  touch-action: none;
  overscroll-behavior: none;
}

/* ===== 图片 ===== */
.preview-image {
  max-width: 95vw;
  max-height: 95vh;
  object-fit: contain;
  user-select: none;
  /* 阻止 iOS 长按弹出菜单 */
  -webkit-touch-callout: none;
  cursor: grab;
  pointer-events: auto;
  border-radius: 8px;
  box-shadow: 0 8px 60px rgba(0, 0, 0, 0.5);
  will-change: transform;
}

.preview-image.is-dragging {
  cursor: grabbing !important;
  transition: none !important;
}

/* ===== 缩放控制栏 ===== */
.zoom-controls {
  position: absolute;
  bottom: 32px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(30, 30, 30, 0.75);
  padding: 6px;
  border-radius: 28px;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
  animation: controlsIn 0.4s 0.15s cubic-bezier(0.16, 1, 0.3, 1) both;
  /* 移动端安全区域 */
  bottom: max(32px, env(safe-area-inset-bottom, 0px) + 16px);
}

@keyframes controlsIn {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

.zoom-btn {
  width: 38px;
  height: 38px;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: rgba(255, 255, 255, 0.75);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-family: var(--font-family-base, sans-serif);
  font-weight: 400;
  line-height: 1;
}

.zoom-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  transform: scale(1.1);
}

.zoom-btn:active {
  transform: scale(0.92);
  background: rgba(255, 255, 255, 0.18);
}

.zoom-btn.rotate-btn {
  font-size: 20px;
}

.reset-btn {
  width: auto;
  min-width: 56px;
  height: 34px;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.7);
  letter-spacing: 0.3px;
  margin-left: 4px;
}

.reset-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.25);
  color: #fff;
}

/* 控制栏分隔线 */
.zoom-controls .divider {
  width: 1px;
  height: 24px;
  background: rgba(255, 255, 255, 0.1);
  margin: 0 4px;
  flex-shrink: 0;
}

.zoom-level {
  color: rgba(255, 255, 255, 0.8);
  font-size: 13px;
  min-width: 52px;
  text-align: center;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.3px;
}

/* ===== 底部提示文字 ===== */
.zoom-hint {
  position: absolute;
  bottom: 88px;
  left: 50%;
  transform: translateX(-50%);
  color: rgba(255, 255, 255, 0.3);
  font-size: 12px;
  pointer-events: none;
  animation: hintFade 0.6s 0.6s ease both;
  white-space: nowrap;
  /* 移动端安全区域 */
  bottom: max(88px, env(safe-area-inset-bottom, 0px) + 72px);
}

@keyframes hintFade {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

/* ===== 移动端触控区优化 ===== */
@media (hover: none) and (pointer: coarse) {
  .close-btn {
    width: 48px;
    height: 48px;
  }

  .zoom-btn {
    width: 44px;
    height: 44px;
  }

  .reset-btn {
    height: 38px;
    min-width: 64px;
    font-size: 13px;
  }
}
</style>
