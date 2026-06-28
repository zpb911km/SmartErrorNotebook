<template>
  <div v-if="visible" class="image-preview-overlay" @click="handleClose">
    <div class="image-preview-container" @click.stop>
      <!-- 关闭按钮 -->
      <button class="close-btn" @click="handleClose" title="关闭">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>

      <!-- 图片 -->
      <div class="image-wrapper" @wheel.capture="handleWheel">
        <img
          :src="imageUrl"
          alt="图片预览"
          :class="['preview-image', { 'is-dragging': isDragging }]"
          :style="{
            transform: `scale(${scale}) rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            transition: isDragging ? 'none' : 'transform 0.1s ease-out'
          }"
          @mousedown="handleMouseDown"
          @dblclick="resetAll"
        />
      </div>

      <!-- 缩放控制 -->
      <div class="zoom-controls">
        <button class="zoom-btn" @click="zoomIn" title="放大">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" width="18" height="18">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
        <span class="zoom-level">{{ Math.round(scale * 100) }}%</span>
        <button class="zoom-btn" @click="zoomOut" title="缩小">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" width="18" height="18">
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
        <span class="divider"></span>
        <button class="zoom-btn rotate-btn" @click="rotateImage" title="旋转">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
            <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16" />
          </svg>
        </button>
        <button class="zoom-btn reset-btn" @click="resetAll" title="重置">
          重置
        </button>
      </div>

      <!-- 提示 -->
      <div class="zoom-hint">滚轮缩放 · 拖拽平移 · 双击重置</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  visible: boolean
  imageUrl: string
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// 缩放比例
const scale = ref(1)

// 平移偏移量
const translateX = ref(0)
const translateY = ref(0)

// 旋转角度
const rotation = ref(0)

// 拖拽状态
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const initialTranslateX = ref(0)
const initialTranslateY = ref(0)

// 计算容器尺寸（用于限制拖拽范围）
const containerSize = ref({ width: 0, height: 0 })
const imageContainer = ref<HTMLDivElement | null>(null)

// 限制拖拽范围，防止图片完全移出可视区域
const clampToViewport = () => {
  if (!imageContainer.value) return

  // 获取容器尺寸
  const rect = imageContainer.value.getBoundingClientRect()
  containerSize.value = {
    width: rect.width,
    height: rect.height
  }

  // 图片渲染后的实际尺寸（需要根据 viewBox 和 object-fit 估算）
  // 这里使用一个简化的方法：基于 max-width/max-height: 95%
  const maxWidth = window.innerWidth * 0.95
  const maxHeight =
    window.innerHeight *
    0.95 *
    ((window.innerHeight * 0.95) / window.innerWidth || 1)

  // 当缩小时，限制移动范围
  if (scale.value < 1) {
    const maxX = (maxWidth - maxWidth * scale.value) / 2
    const maxY = (maxHeight - maxHeight * scale.value) / 2
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

// 在组件挂载后和 visible 变化时计算视图口限制
watch(
  () => props.visible,
  () => {
    if (props.visible) {
      // 延迟一下确保 DOM 已渲染
      setTimeout(clampToViewport, 100)
    }
  }
)

// 监听 visible 变化，重置缩放和平移
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      scale.value = 1
      translateX.value = 0
      translateY.value = 0
      rotation.value = 0
    }
  }
)

// 关闭预览
const handleClose = () => {
  emit('close')
}

// 放大
const zoomIn = () => {
  scale.value = Math.min(scale.value * 1.2, 5)
}

// 缩小
const zoomOut = () => {
  scale.value = Math.max(scale.value / 1.2, 0.1)
}

// 重置所有状态
const resetAll = () => {
  scale.value = 1
  translateX.value = 0
  translateY.value = 0
  rotation.value = 0
}

// 旋转图片（90 度）
const rotateImage = () => {
  rotation.value = (rotation.value + 90) % 360
}

// 滚轮缩放（以屏幕中心为中心）
const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  e.stopPropagation()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  scale.value = Math.max(0.1, Math.min(5, scale.value * delta))
}

// 鼠标按下开始拖拽
const handleMouseDown = (e: MouseEvent) => {
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

    // 缩小时限制拖拽范围
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
}

@keyframes overlayIn {
  from { opacity: 0; }
  to { opacity: 1; }
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

/* ===== 图片 ===== */
.preview-image {
  max-width: 95vw;
  max-height: 95vh;
  object-fit: contain;
  user-select: none;
  cursor: grab;
  pointer-events: auto;
  border-radius: 4px;
  box-shadow: 0 4px 40px rgba(0, 0, 0, 0.3);
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
}

@keyframes hintFade {
  from { opacity: 0; transform: translateX(-50%) translateY(8px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
