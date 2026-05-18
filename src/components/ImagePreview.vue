<template>
  <div v-if="visible" class="image-preview-overlay" @click="handleClose">
    <div class="image-preview-container" @click.stop>
      <!-- 关闭按钮 -->
      <button class="close-btn" @click="handleClose" title="关闭">
        <span class="close-icon">x</span>
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
        <button class="zoom-btn" @click="zoomIn" title="放大">+</button>
        <span class="zoom-level">{{ Math.round(scale * 100) }}%</span>
        <button class="zoom-btn" @click="zoomOut" title="缩小">−</button>
        <button class="zoom-btn rotate-btn" @click="rotateImage" title="旋转">
          ↻
        </button>
        <button class="zoom-btn reset-btn" @click="resetAll" title="重置">
          重置
        </button>
      </div>
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
.image-preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.image-preview-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 48px;
  height: 48px;
  background: rgba(224, 224, 224, 0.4);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:focus {
  outline: none;
  box-shadow: none;
}

.close-btn:hover {
  background: rgba(208, 208, 208, 0.7);
  transform: scale(1.1);
}

.close-btn:hover .close-icon {
  color: red;
  transform: scale(1.5);
}

.close-icon {
  font-size: 32px;
  color: black;
  transition: all 0.3s ease;
  position: relative;
  top: -5px;
}

.preview-image {
  width: 100vw;
  height: 100vh;
  object-fit: contain;
  user-select: none;
  cursor: grab;
  pointer-events: auto;
}

.preview-image.is-dragging {
  cursor: grabbing !important;
  transition: none !important;
}

#imagePreviewImg:active {
  cursor: grabbing !important;
}

.zoom-controls {
  position: absolute;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(0, 0, 0, 0.6);
  padding: 8px 16px;
  border-radius: 24px;
  backdrop-filter: blur(10px);
}

.zoom-btn {
  width: 36px;
  height: 36px;
  background: rgba(255, 255, 255, 0.2);
  border: none;
  border-radius: 50%;
  color: black;
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s;
}

.zoom-btn.rotate-btn {
  border-radius: 50%;
}

.reset-btn {
  width: auto;
  min-width: 56px;
  height: 36px;
  padding: 0 12px;
  font-size: 14px;
  font-weight: bold;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  gap: 4px;
}

.zoom-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.1);
}

.zoom-level {
  color: white;
  font-size: 14px;
  min-width: 60px;
  text-align: center;
  font-weight: 500;
}
</style>
