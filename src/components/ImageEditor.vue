<template>
  <div class="edit-modal" v-if="visible">
    <div class="edit-header">
      <button class="header-btn close-btn" @click="handleCancel">
        <Icon name="x" :size="18" />
      </button>
      <span class="edit-title">图片编辑</span>
      <button class="header-btn confirm-btn" @click="handleConfirm">✓</button>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <button
        v-for="tool in tools"
        :key="tool.id"
        class="tool-btn"
        :class="{ active: currentTool === tool.id }"
        @click="selectTool(tool.id)"
        :title="tool.name"
      >
        {{ tool.icon }}
      </button>

      <div class="toolbar-divider"></div>

      <button
        class="tool-btn"
        @click="undo"
        :disabled="historyIndex <= 0"
        title="撤销"
      >
        ↩️
      </button>
      <button
        class="tool-btn"
        @click="redo"
        :disabled="historyIndex >= history.length - 1"
        title="重做"
      >
        ↪️
      </button>

      <div class="toolbar-divider"></div>

      <button class="tool-btn" @click="resetAll" title="重置">
        <Icon name="refresh-cw" :size="16" />
      </button>
    </div>

    <!-- 调整面板 -->
    <div class="adjustment-panel" v-if="showAdjustmentPanel">
      <div class="panel-header">
        <span>{{ currentToolName }}</span>
        <button class="panel-close" @click="showAdjustmentPanel = false">
          <Icon name="x" :size="16" />
        </button>
      </div>

      <div class="panel-content">
        <!-- 裁剪工具 -->
        <div v-if="currentTool === 'crop'" class="crop-controls">
          <p class="hint">拖动四个角点框选裁剪区域</p>
          <div class="crop-buttons">
            <button class="control-btn" @click="resetCrop">重置选区</button>
            <button class="control-btn primary" @click="applyCrop">
              应用裁剪
            </button>
          </div>
        </div>

        <!-- 旋转工具 -->
        <div v-if="currentTool === 'rotate'" class="rotate-controls">
          <button class="control-btn" @click="rotate(-90)">↺ 左转90°</button>
          <button class="control-btn" @click="rotate(90)">↻ 右转90°</button>
          <button class="control-btn" @click="rotate(180)">↔ 180°</button>
          <div class="slider-control">
            <label>自由旋转: {{ rotationAngle }}°</label>
            <input
              type="range"
              v-model.number="rotationAngle"
              min="-180"
              max="180"
              @input="previewRotation"
              @change="applyRotation"
            />
          </div>
        </div>

        <!-- 对比度工具 -->
        <div v-if="currentTool === 'contrast'" class="contrast-controls">
          <div class="slider-control">
            <label>对比度: {{ contrast }}%</label>
            <input
              type="range"
              v-model.number="contrast"
              min="0"
              max="200"
              @input="previewContrast"
            />
          </div>
          <div class="slider-control">
            <label>亮度: {{ brightness }}%</label>
            <input
              type="range"
              v-model.number="brightness"
              min="0"
              max="200"
              @input="previewContrast"
            />
          </div>
          <button class="control-btn" @click="resetContrast">重置</button>
          <button class="control-btn primary" @click="applyContrast">
            应用
          </button>
        </div>

        <!-- 二值化工具 -->
        <div v-if="currentTool === 'threshold'" class="threshold-controls">
          <div class="slider-control">
            <label>阈值: {{ threshold }}</label>
            <input
              type="range"
              v-model.number="threshold"
              min="0"
              max="255"
              @input="previewThreshold"
            />
          </div>
          <div class="checkbox-control">
            <input type="checkbox" id="invert" v-model="invertThreshold" />
            <label for="invert">反色</label>
          </div>
          <button class="control-btn" @click="resetThreshold">重置</button>
          <button class="control-btn primary" @click="applyThreshold">
            应用
          </button>
        </div>
      </div>
    </div>

    <div class="edit-canvas-container" ref="containerRef">
      <canvas
        ref="canvasRef"
        @mousedown="handleMouseDown"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
        @wheel="handleWheel"
      ></canvas>
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <span>{{ imageInfo }}</span>
      <span v-if="currentTool === 'crop'">{{ cropInfo }}</span>
    </div>

    <!-- 放大镜 -->
    <div
      class="loupe"
      v-if="showLoupe"
      :style="{ left: loupePosition.x + 'px', top: loupePosition.y + 'px' }"
    >
      <canvas ref="loupeCanvasRef" width="150" height="150"></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue'

interface Props {
  visible: boolean
  imageData: string
  autoDetect?: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'confirm', imageData: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// 工具定义
const tools = [
  { id: 'crop', name: '裁剪', icon: '✂️' },
  { id: 'rotate', name: '旋转', icon: '🔄' },
  { id: 'contrast', name: '对比度', icon: '🌓' },
  { id: 'threshold', name: '二值化', icon: '◐' }
]

// Canvas 引用
const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)

// 图像数据
const originalImage = ref<HTMLImageElement | null>(null)
const currentImage = ref<HTMLImageElement | null>(null)

// 操作历史
const history = ref<string[]>([])
const historyIndex = ref(-1)

// 当前工具
const currentTool = ref<string>('crop')
const showAdjustmentPanel = ref(true)

// 裁剪相关
const corners = ref<{ x: number; y: number }[]>([
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 }
])
const draggingCorner = ref<number | null>(null)
// 旋转相关
const rotationAngle = ref(0)

// 对比度相关
const contrast = ref(100)
const brightness = ref(100)

// 二值化相关
const threshold = ref(128)
const invertThreshold = ref(false)

// 缩放相关
const scale = ref(1)
const panX = ref(0)
const panY = ref(0)
const isPanning = ref(false)
const lastPanPos = ref({ x: 0, y: 0 })

// 边缘拖拽相关
const draggingEdge = ref<number | null>(null)
const lastEdgePos = ref({ x: 0, y: 0 })

// 放大镜相关
const showLoupe = ref(false)
const loupePosition = ref({ x: 0, y: 0 })
const loupeCanvasRef = ref<HTMLCanvasElement | null>(null)
const touchScreenPos = ref({ x: 0, y: 0 })

// 双指缩放相关
const isPinching = ref(false)
const initialPinchDistance = ref(0)
const initialPinchScale = ref(1)

// 计算属性
const currentToolName = computed(() => {
  const tool = tools.find((t) => t.id === currentTool.value)
  return tool ? tool.name : ''
})

const imageInfo = computed(() => {
  if (!currentImage.value) return ''
  return `${currentImage.value.width} × ${currentImage.value.height}px`
})

const cropInfo = computed(() => {
  if (
    currentTool.value !== 'crop' ||
    corners.value.every((c) => c.x === 0 && c.y === 0)
  ) {
    return ''
  }
  const minX = Math.min(...corners.value.map((c) => c.x))
  const minY = Math.min(...corners.value.map((c) => c.y))
  const maxX = Math.max(...corners.value.map((c) => c.x))
  const maxY = Math.max(...corners.value.map((c) => c.y))
  return `选区: ${Math.round(maxX - minX)} × ${Math.round(maxY - minY)}px`
})

// 监听 visible 变化
watch(
  () => props.visible,
  async (newVal) => {
    console.log(
      'ImageEditor visible changed:',
      newVal,
      'autoDetect:',
      props.autoDetect
    )
    if (newVal && props.imageData) {
      await loadImage()
    }
  }
)

// 加载图片
const loadImage = () => {
  console.log('ImageEditor loadImage, autoDetect:', props.autoDetect)
  const img = new Image()
  img.onload = () => {
    originalImage.value = img
    currentImage.value = img

    // 初始化历史记录
    history.value = [props.imageData]
    historyIndex.value = 0

    // 重置所有参数
    resetAllParameters()

    nextTick(() => {
      initCanvas()
      initCropCorners()

      // 如果需要自动识别，延迟执行以确保Canvas初始化完成
      if (props.autoDetect) {
        console.log('自动识别开始执行')
        setTimeout(() => {
          autoDetectRegion()
        }, 500)
      }
    })
  }
  img.src = props.imageData
}

// 初始化 Canvas
const initCanvas = () => {
  if (!canvasRef.value || !containerRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const container = containerRef.value
  const img = currentImage.value

  // 设置 Canvas 尺寸
  const maxWidth = container.clientWidth
  const maxHeight = container.clientHeight

  const imgScale = Math.min(maxWidth / img.width, maxHeight / img.height, 1)
  scale.value = imgScale
  panX.value = 0
  panY.value = 0

  canvas.width = img.width
  canvas.height = img.height

  drawCanvas()
}

// 初始化裁剪角点
const initCropCorners = () => {
  if (!canvasRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const padding = 0

  corners.value = [
    { x: padding, y: padding },
    { x: canvas.width - padding, y: padding },
    { x: canvas.width - padding, y: canvas.height - padding },
    { x: padding, y: canvas.height - padding }
  ]
}

// 绘制 Canvas
const drawCanvas = () => {
  if (!canvasRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')

  if (!ctx) return

  const img = currentImage.value

  // 清空 Canvas
  ctx.clearRect(0, 0, canvas.width, canvas.height)

  // 绘制图片
  ctx.drawImage(img, 0, 0, canvas.width, canvas.height)

  // 如果是裁剪工具或自动识别工具，绘制裁剪框
  if (currentTool.value === 'crop' || currentTool.value === 'auto-detect') {
    drawCropOverlay(ctx, canvas)
  }
}

// 绘制裁剪遮罩
const drawCropOverlay = (
  ctx: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement
) => {
  // 绘制半透明遮罩
  ctx.fillStyle = 'rgba(0, 0, 0, 0.5)'
  ctx.fillRect(0, 0, canvas.width, canvas.height)

  // 创建裁剪区域（四边形内部不遮罩）
  ctx.save()
  ctx.beginPath()
  ctx.moveTo(corners.value[0].x, corners.value[0].y)
  ctx.lineTo(corners.value[1].x, corners.value[1].y)
  ctx.lineTo(corners.value[2].x, corners.value[2].y)
  ctx.lineTo(corners.value[3].x, corners.value[3].y)
  ctx.closePath()
  ctx.clip()
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.drawImage(currentImage.value!, 0, 0, canvas.width, canvas.height)
  ctx.restore()

  // 绘制四边形边框（动态线宽）
  ctx.strokeStyle = '#1976d2'
  ctx.lineWidth = getDynamicLineWidth(3)
  ctx.beginPath()
  ctx.moveTo(corners.value[0].x, corners.value[0].y)
  ctx.lineTo(corners.value[1].x, corners.value[1].y)
  ctx.lineTo(corners.value[2].x, corners.value[2].y)
  ctx.lineTo(corners.value[3].x, corners.value[3].y)
  ctx.closePath()
  ctx.stroke()

  // 绘制顶点（动态半径）
  const cornerRadius = getDynamicCornerRadius()
  corners.value.forEach((corner) => {
    ctx.beginPath()
    ctx.arc(corner.x, corner.y, cornerRadius, 0, Math.PI * 2)
    ctx.fillStyle = '#1976d2'
    ctx.fill()
    ctx.strokeStyle = 'white'
    ctx.lineWidth = getDynamicLineWidth(2)
    ctx.stroke()
  })

  // 绘制边缘中点菱形（示意可拖拽）
  const edgeList: [number, number][] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0]
  ]
  const diamondSize = cornerRadius * 0.6
  const diamondLineWidth = getDynamicLineWidth(1.5)
  edgeList.forEach(([i1, i2]) => {
    const mx = (corners.value[i1].x + corners.value[i2].x) / 2
    const my = (corners.value[i1].y + corners.value[i2].y) / 2

    ctx.beginPath()
    ctx.moveTo(mx, my - diamondSize)
    ctx.lineTo(mx + diamondSize, my)
    ctx.lineTo(mx, my + diamondSize)
    ctx.lineTo(mx - diamondSize, my)
    ctx.closePath()
    ctx.fillStyle = '#1976d2'
    ctx.fill()
    ctx.strokeStyle = 'white'
    ctx.lineWidth = diamondLineWidth
    ctx.stroke()
  })

  // 绘制辅助参考线（拖拽角点时显示）
  if (draggingCorner.value !== null) {
    const dragged = corners.value[draggingCorner.value]
    const guideWidth = getDynamicLineWidth(1.5)
    const avgScale = (getCanvasScale().scaleX + getCanvasScale().scaleY) / 2

    ctx.save()
    ctx.strokeStyle = 'rgba(100, 180, 255, 0.65)'
    ctx.lineWidth = guideWidth
    ctx.setLineDash([Math.max(4, 8 * avgScale), Math.max(2, 4 * avgScale)])

    ctx.beginPath()
    ctx.moveTo(0, dragged.y)
    ctx.lineTo(canvas.width, dragged.y)
    ctx.stroke()

    ctx.beginPath()
    ctx.moveTo(dragged.x, 0)
    ctx.lineTo(dragged.x, canvas.height)
    ctx.stroke()

    ctx.restore()
  }
}

// 获取 Canvas 的缩放比例
const getCanvasScale = () => {
  if (!canvasRef.value) return { scaleX: 1, scaleY: 1 }

  const rect = canvasRef.value.getBoundingClientRect()
  if (rect.width === 0 || rect.height === 0) return { scaleX: 1, scaleY: 1 }
  const scaleX = canvasRef.value.width / rect.width
  const scaleY = canvasRef.value.height / rect.height

  return { scaleX, scaleY }
}

// 将屏幕坐标转换为 Canvas 内部坐标
const screenToCanvas = (screenX: number, screenY: number) => {
  const rect = canvasRef.value?.getBoundingClientRect()
  if (!rect) return { x: 0, y: 0 }

  const { scaleX, scaleY } = getCanvasScale()

  const x = (screenX - rect.left) * scaleX
  const y = (screenY - rect.top) * scaleY

  return { x, y }
}

// 获取动态触控半径（屏幕像素 28px 转 canvas 坐标）
const getDynamicTouchRadius = () => {
  const { scaleX, scaleY } = getCanvasScale()
  return (28 * (scaleX + scaleY)) / 2
}

// 获取动态角点半径（屏幕像素 14px 转 canvas 坐标）
const getDynamicCornerRadius = () => {
  const { scaleX, scaleY } = getCanvasScale()
  return Math.max(6, (14 * (scaleX + scaleY)) / 2)
}

// 获取动态线宽（屏幕像素转 canvas 坐标）
const getDynamicLineWidth = (targetScreenPx: number) => {
  const { scaleX, scaleY } = getCanvasScale()
  return Math.max(1, (targetScreenPx * (scaleX + scaleY)) / 2)
}

// 点到线段的最短距离
const pointToSegmentDistance = (
  p: { x: number; y: number },
  a: { x: number; y: number },
  b: { x: number; y: number }
) => {
  const abx = b.x - a.x
  const aby = b.y - a.y
  const apx = p.x - a.x
  const apy = p.y - a.y
  const ab2 = abx * abx + aby * aby
  if (ab2 === 0) return Math.sqrt(apx * apx + apy * apy)
  let t = (apx * abx + apy * aby) / ab2
  t = Math.max(0, Math.min(1, t))
  const projx = a.x + t * abx
  const projy = a.y + t * aby
  return Math.sqrt((p.x - projx) ** 2 + (p.y - projy) ** 2)
}

// 查找最近的边缘（四边形的四条边）
const findNearestEdge = (canvasX: number, canvasY: number): number | null => {
  const edgeList: [number, number][] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0]
  ]
  const radius = getDynamicTouchRadius()

  for (let i = 0; i < 4; i++) {
    const [i1, i2] = edgeList[i]
    const dist = pointToSegmentDistance(
      { x: canvasX, y: canvasY },
      corners.value[i1],
      corners.value[i2]
    )
    if (dist <= radius) return i
  }
  return null
}

// 全局鼠标事件处理函数
const handleGlobalMouseMove = (e: MouseEvent) => {
  if (currentTool.value === 'crop') {
    if (draggingCorner.value !== null) {
      const { x, y } = screenToCanvas(e.clientX, e.clientY)

      if (canvasRef.value) {
        const width = canvasRef.value.width
        const height = canvasRef.value.height

        corners.value[draggingCorner.value] = {
          x: Math.max(0, Math.min(x, width)),
          y: Math.max(0, Math.min(y, height))
        }
        drawCanvas()
      }
    } else if (draggingEdge.value !== null) {
      const { x, y } = screenToCanvas(e.clientX, e.clientY)
      const edgeList: [number, number][] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0]
      ]
      const [i1, i2] = edgeList[draggingEdge.value]

      const dx = x - lastEdgePos.value.x
      const dy = y - lastEdgePos.value.y

      if (canvasRef.value) {
        const w = canvasRef.value.width
        const h = canvasRef.value.height

        // 约束 dx/dy 使两个角点同时不超过边界，避免边缩短
        const minDx = Math.max(-corners.value[i1].x, -corners.value[i2].x)
        const maxDx = Math.min(w - corners.value[i1].x, w - corners.value[i2].x)
        const constrainedDx = Math.max(minDx, Math.min(dx, maxDx))

        const minDy = Math.max(-corners.value[i1].y, -corners.value[i2].y)
        const maxDy = Math.min(h - corners.value[i1].y, h - corners.value[i2].y)
        const constrainedDy = Math.max(minDy, Math.min(dy, maxDy))

        corners.value[i1] = {
          x: corners.value[i1].x + constrainedDx,
          y: corners.value[i1].y + constrainedDy
        }
        corners.value[i2] = {
          x: corners.value[i2].x + constrainedDx,
          y: corners.value[i2].y + constrainedDy
        }
      }

      lastEdgePos.value = { x, y }
      drawCanvas()
    }
  } else if (isPanning.value) {
    const dx = e.clientX - lastPanPos.value.x
    const dy = e.clientY - lastPanPos.value.y

    panX.value += dx
    panY.value += dy

    lastPanPos.value = { x: e.clientX, y: e.clientY }

    updateCanvasTransform()
  }
}

const handleGlobalMouseUp = () => {
  window.removeEventListener('mousemove', handleGlobalMouseMove)
  window.removeEventListener('mouseup', handleGlobalMouseUp)

  if (canvasRef.value) {
    const width = canvasRef.value.width
    const height = canvasRef.value.height

    corners.value = corners.value.map((corner) => ({
      x: Math.max(0, Math.min(corner.x, width)),
      y: Math.max(0, Math.min(corner.y, height))
    }))

    drawCanvas()
  }

  draggingCorner.value = null
  draggingEdge.value = null
  isPanning.value = false
}

// 鼠标事件处理
const handleMouseDown = (e: MouseEvent) => {
  if (currentTool.value === 'crop') {
    const { x, y } = screenToCanvas(e.clientX, e.clientY)
    const radius = getDynamicTouchRadius()

    // 检查角点（优先于边缘）
    let hit = false
    corners.value.forEach((corner, index) => {
      const distance = Math.sqrt((x - corner.x) ** 2 + (y - corner.y) ** 2)
      if (distance <= radius && !hit) {
        draggingCorner.value = index
        hit = true
      }
    })

    // 检查边缘
    if (!hit) {
      const edgeIndex = findNearestEdge(x, y)
      if (edgeIndex !== null) {
        draggingEdge.value = edgeIndex
        lastEdgePos.value = { x, y }
        hit = true
      }
    }

    if (hit) {
      window.addEventListener('mousemove', handleGlobalMouseMove)
      window.addEventListener('mouseup', handleGlobalMouseUp)
    }
  } else {
    isPanning.value = true
    lastPanPos.value = { x: e.clientX, y: e.clientY }
    window.addEventListener('mousemove', handleGlobalMouseMove)
    window.addEventListener('mouseup', handleGlobalMouseUp)
  }
}

// 触摸事件处理
const handleTouchStart = (e: TouchEvent) => {
  e.preventDefault()

  // 双指缩放检测（优先于所有单指操作）
  if (e.touches.length === 2) {
    // 如果之前有裁剪拖拽，先清理
    draggingCorner.value = null
    draggingEdge.value = null
    showLoupe.value = false
    // 记录双指初始距离
    const dx = e.touches[0].clientX - e.touches[1].clientX
    const dy = e.touches[0].clientY - e.touches[1].clientY
    initialPinchDistance.value = Math.sqrt(dx * dx + dy * dy)
    initialPinchScale.value = scale.value
    isPinching.value = true
    return
  }

  if (currentTool.value === 'crop') {
    const touch = e.touches[0]
    const { x, y } = screenToCanvas(touch.clientX, touch.clientY)
    const radius = getDynamicTouchRadius()

    touchScreenPos.value = { x: touch.clientX, y: touch.clientY }

    // 检查角点（优先于边缘）
    let hit = false
    corners.value.forEach((corner, index) => {
      const distance = Math.sqrt((x - corner.x) ** 2 + (y - corner.y) ** 2)
      if (distance <= radius && !hit) {
        draggingCorner.value = index
        hit = true
        showLoupe.value = true
      }
    })

    // 检查边缘
    if (!hit) {
      const edgeIndex = findNearestEdge(x, y)
      if (edgeIndex !== null) {
        draggingEdge.value = edgeIndex
        lastEdgePos.value = { x, y }
        hit = true
      }
    }

    if (hit) {
      updateLoupePosition(touch.clientX, touch.clientY)
      requestAnimationFrame(() => updateLoupeContent())
    }
  }
}

const handleTouchMove = (e: TouchEvent) => {
  e.preventDefault()

  // 双指缩放
  if (isPinching.value && e.touches.length === 2) {
    const dx = e.touches[0].clientX - e.touches[1].clientX
    const dy = e.touches[0].clientY - e.touches[1].clientY
    const distance = Math.sqrt(dx * dx + dy * dy)
    const ratio = distance / initialPinchDistance.value
    scale.value = Math.max(0.1, Math.min(5, initialPinchScale.value * ratio))
    updateCanvasTransform()
    return
  }

  const touch = e.touches[0]
  touchScreenPos.value = { x: touch.clientX, y: touch.clientY }

  if (currentTool.value === 'crop') {
    if (draggingCorner.value !== null) {
      const { x, y } = screenToCanvas(touch.clientX, touch.clientY)

      if (canvasRef.value) {
        const width = canvasRef.value.width
        const height = canvasRef.value.height

        corners.value[draggingCorner.value] = {
          x: Math.max(0, Math.min(x, width)),
          y: Math.max(0, Math.min(y, height))
        }
        drawCanvas()
        updateLoupePosition(touch.clientX, touch.clientY)
        requestAnimationFrame(() => updateLoupeContent())
      }
    } else if (draggingEdge.value !== null) {
      const { x, y } = screenToCanvas(touch.clientX, touch.clientY)
      const edgeList: [number, number][] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0]
      ]
      const [i1, i2] = edgeList[draggingEdge.value]

      const dx = x - lastEdgePos.value.x
      const dy = y - lastEdgePos.value.y

      if (canvasRef.value) {
        const w = canvasRef.value.width
        const h = canvasRef.value.height

        // 约束 dx/dy 使两个角点同时不超过边界，避免边缩短
        const minDx = Math.max(-corners.value[i1].x, -corners.value[i2].x)
        const maxDx = Math.min(w - corners.value[i1].x, w - corners.value[i2].x)
        const constrainedDx = Math.max(minDx, Math.min(dx, maxDx))

        const minDy = Math.max(-corners.value[i1].y, -corners.value[i2].y)
        const maxDy = Math.min(h - corners.value[i1].y, h - corners.value[i2].y)
        const constrainedDy = Math.max(minDy, Math.min(dy, maxDy))

        corners.value[i1] = {
          x: corners.value[i1].x + constrainedDx,
          y: corners.value[i1].y + constrainedDy
        }
        corners.value[i2] = {
          x: corners.value[i2].x + constrainedDx,
          y: corners.value[i2].y + constrainedDy
        }
      }

      lastEdgePos.value = { x, y }
      drawCanvas()
    }
  }
}

const handleTouchEnd = (e: TouchEvent) => {
  // 双指缩放结束检测
  if (isPinching.value && e.touches.length < 2) {
    isPinching.value = false
  }

  if (canvasRef.value) {
    const width = canvasRef.value.width
    const height = canvasRef.value.height

    corners.value = corners.value.map((corner) => ({
      x: Math.max(0, Math.min(corner.x, width)),
      y: Math.max(0, Math.min(corner.y, height))
    }))

    drawCanvas()
  }

  draggingCorner.value = null
  draggingEdge.value = null
  showLoupe.value = false
}

// 更新放大镜位置（自动避开屏幕边缘）
const updateLoupePosition = (clientX: number, clientY: number) => {
  const loupeSize = 150
  const margin = 12
  const offsetAbove = 50
  const offsetBelow = 30

  let left: number
  let top: number

  // 垂直方向：优先显示在手指上方，空间不够则显示在下方
  if (clientY - offsetAbove - loupeSize - margin > 0) {
    top = clientY - offsetAbove - loupeSize
  } else {
    top = clientY + offsetBelow
  }

  // 水平方向：略偏右居中，贴近边缘时左移
  left = clientX - loupeSize / 2 + 10
  left = Math.max(
    margin,
    Math.min(left, window.innerWidth - loupeSize - margin)
  )

  // 如果上方显示空间不够同时下方也超出底部，强制上移
  if (
    top + loupeSize >
    (window.innerHeight || document.documentElement.clientHeight) - margin
  ) {
    top =
      (window.innerHeight || document.documentElement.clientHeight) -
      loupeSize -
      margin
  }

  loupePosition.value = { x: left, y: top }
}

// 绘制放大镜内容
const updateLoupeContent = () => {
  if (
    !loupeCanvasRef.value ||
    !currentImage.value ||
    draggingCorner.value === null
  )
    return

  const lCanvas = loupeCanvasRef.value
  const lCtx = lCanvas.getContext('2d')
  if (!lCtx) return

  const loupeSize = 150
  const magnification = 2.5
  const regionSize = loupeSize / magnification // 60px canvas 坐标区域

  const corner = corners.value[draggingCorner.value]
  const halfRegion = regionSize / 2

  const srcX = Math.max(0, corner.x - halfRegion)
  const srcY = Math.max(0, corner.y - halfRegion)
  const srcW = Math.min(
    regionSize,
    (canvasRef.value?.width ?? regionSize) - srcX
  )
  const srcH = Math.min(
    regionSize,
    (canvasRef.value?.height ?? regionSize) - srcY
  )

  // 清空 + 白色背景
  lCtx.fillStyle = '#ffffff'
  lCtx.fillRect(0, 0, loupeSize, loupeSize)

  // 绘制放大后的图像区域
  if (currentImage.value) {
    lCtx.drawImage(
      currentImage.value,
      srcX,
      srcY,
      srcW,
      srcH,
      0,
      0,
      loupeSize,
      loupeSize
    )
  }

  // 计算角点在放大镜 canvas 中的位置
  const cx = ((corner.x - srcX) / regionSize) * loupeSize
  const cy = ((corner.y - srcY) / regionSize) * loupeSize

  // 十字准星（红色）
  lCtx.strokeStyle = '#ff4444'
  lCtx.lineWidth = 1.5
  lCtx.beginPath()
  lCtx.moveTo(cx - 20, cy)
  lCtx.lineTo(cx + 20, cy)
  lCtx.stroke()
  lCtx.beginPath()
  lCtx.moveTo(cx, cy - 20)
  lCtx.lineTo(cx, cy + 20)
  lCtx.stroke()

  // 中心蓝色角点标记
  lCtx.beginPath()
  lCtx.arc(cx, cy, 4, 0, Math.PI * 2)
  lCtx.fillStyle = '#1976d2'
  lCtx.fill()
  lCtx.strokeStyle = '#ffffff'
  lCtx.lineWidth = 1.5
  lCtx.stroke()
}

// 滚轮缩放
const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  scale.value = Math.max(0.1, Math.min(5, scale.value * delta))
  updateCanvasTransform()
}

// 更新 Canvas 变换
const updateCanvasTransform = () => {
  if (!canvasRef.value) return

  canvasRef.value.style.transform = `scale(${scale.value}) translate(${panX.value / scale.value}px, ${panY.value / scale.value}px)`
}

// 选择工具
const selectTool = (toolId: string) => {
  currentTool.value = toolId
  showAdjustmentPanel.value = true

  // 重置参数
  if (toolId === 'rotate') {
    rotationAngle.value = 0
  } else if (toolId === 'contrast') {
    contrast.value = 100
    brightness.value = 100
  } else if (toolId === 'threshold') {
    threshold.value = 128
    invertThreshold.value = false
  }

  drawCanvas()
}

// 应用阈值到图像
const applyThresholdToImage = (
  imageData: ImageData,
  threshold: number
): ImageData => {
  const data = imageData.data
  const result = new ImageData(imageData.width, imageData.height)
  const resultData = result.data

  for (let i = 0; i < data.length; i += 4) {
    const gray = (data[i] + data[i + 1] + data[i + 2]) / 3
    const value = gray >= threshold ? 255 : 0
    resultData[i] = value
    resultData[i + 1] = value
    resultData[i + 2] = value
    resultData[i + 3] = 255
  }

  return result
}

// Sobel 边缘检测
const sobelEdgeDetection = (imageData: ImageData): ImageData => {
  const data = imageData.data
  const width = imageData.width
  const height = imageData.height
  const result = new ImageData(width, height)
  const resultData = result.data

  // Sobel 算子
  const sobelX = [
    [-1, 0, 1],
    [-2, 0, 2],
    [-1, 0, 1]
  ]

  const sobelY = [
    [-1, -2, -1],
    [0, 0, 0],
    [1, 2, 1]
  ]

  // 遍历图像像素
  for (let y = 1; y < height - 1; y++) {
    for (let x = 1; x < width - 1; x++) {
      let gx = 0
      let gy = 0

      // 应用 Sobel 算子
      for (let i = -1; i <= 1; i++) {
        for (let j = -1; j <= 1; j++) {
          const idx = ((y + i) * width + (x + j)) * 4
          const gray = (data[idx] + data[idx + 1] + data[idx + 2]) / 3
          gx += gray * sobelX[i + 1][j + 1]
          gy += gray * sobelY[i + 1][j + 1]
        }
      }

      // 计算梯度幅值
      const magnitude = Math.sqrt(gx * gx + gy * gy)
      const value = Math.min(255, magnitude)

      const resultIdx = (y * width + x) * 4
      resultData[resultIdx] = value
      resultData[resultIdx + 1] = value
      resultData[resultIdx + 2] = value
      resultData[resultIdx + 3] = 255
    }
  }

  return result
}

// 洪水填充算法
const floodFill = (
  x: number,
  y: number,
  width: number,
  height: number,
  data: Uint8ClampedArray,
  visited: boolean[]
): { x: number; y: number }[] => {
  const stack = [{ x, y }]
  const contour: { x: number; y: number }[] = []
  const directions = [
    [0, 1],
    [1, 0],
    [0, -1],
    [-1, 0]
  ]

  while (stack.length > 0) {
    const { x, y } = stack.pop()!
    const idx = y * width + x

    if (
      x < 0 ||
      x >= width ||
      y < 0 ||
      y >= height ||
      visited[idx] ||
      data[idx * 4] <= 128
    ) {
      continue
    }

    visited[idx] = true
    contour.push({ x, y })

    for (const [dx, dy] of directions) {
      stack.push({ x: x + dx, y: y + dy })
    }
  }

  return contour
}

// 提取轮廓
const findContours = (imageData: ImageData): { x: number; y: number }[] => {
  const data = imageData.data
  const width = imageData.width
  const height = imageData.height
  const visited = new Array(width * height).fill(false)
  const contours: { x: number; y: number }[] = []

  // 简单的轮廓提取算法
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4
      if (data[idx] > 128 && !visited[y * width + x]) {
        // 找到一个轮廓
        const contour = floodFill(x, y, width, height, data, visited)
        if (contour.length > 100) {
          // 过滤小轮廓
          contours.push(...contour)
        }
      }
    }
  }

  return contours
}

// 拟合矩形
const fitRectangle = (
  points: { x: number; y: number }[]
): { x: number; y: number }[] => {
  if (points.length === 0) return []

  const minX = Math.min(...points.map((p) => p.x))
  const minY = Math.min(...points.map((p) => p.y))
  const maxX = Math.max(...points.map((p) => p.x))
  const maxY = Math.max(...points.map((p) => p.y))

  return [
    { x: minX, y: minY },
    { x: maxX, y: minY },
    { x: maxX, y: maxY },
    { x: minX, y: maxY }
  ]
}

// 自动识别区域
const autoDetectRegion = () => {
  console.log('自动识别开始')
  if (!canvasRef.value || !currentImage.value) {
    console.log('Canvas或图像不存在')
    return
  }

  const img = currentImage.value
  console.log('原始尺寸:', img.width, 'x', img.height)

  // 降采样到最长边 800px，加速算法执行
  const MAX_DIM = 800
  const scale = Math.min(MAX_DIM / img.width, MAX_DIM / img.height, 1)
  const smallW = Math.round(img.width * scale)
  const smallH = Math.round(img.height * scale)

  const tempCanvas = document.createElement('canvas')
  tempCanvas.width = smallW
  tempCanvas.height = smallH
  const tempCtx = tempCanvas.getContext('2d')
  if (!tempCtx) {
    console.log('无法创建临时Canvas')
    return
  }

  // 浏览器原生缩放（C++ 层面，不会卡）
  tempCtx.drawImage(img, 0, 0, smallW, smallH)

  // 缩略图上跑算法
  const imageData = tempCtx.getImageData(0, 0, smallW, smallH)
  console.log('降采样后尺寸:', smallW, 'x', smallH, '缩放比:', scale)

  const thresholdedData = applyThresholdToImage(imageData, 128)
  console.log('二值化处理完成')

  const edgeData = sobelEdgeDetection(thresholdedData)
  console.log('边缘检测完成')

  const contours = findContours(edgeData)
  console.log('提取轮廓完成，轮廓点数量:', contours.length)

  const rectangle = fitRectangle(contours)
  console.log('拟合矩形完成，矩形点数量:', rectangle.length)

  if (rectangle.length === 4) {
    console.log('识别到区域:', rectangle)
    // 将缩略图坐标映射回原图坐标
    const invScale = 1 / scale
    const width = canvasRef.value.width
    const height = canvasRef.value.height
    const padding = 20

    corners.value = rectangle.map((point, index) => {
      const origX = point.x * invScale
      const origY = point.y * invScale
      if (index === 0) {
        return {
          x: Math.max(0, origX - padding),
          y: Math.max(0, origY - padding)
        }
      } else if (index === 1) {
        return {
          x: Math.min(width, origX + padding),
          y: Math.max(0, origY - padding)
        }
      } else if (index === 2) {
        return {
          x: Math.min(width, origX + padding),
          y: Math.min(height, origY + padding)
        }
      } else {
        return {
          x: Math.max(0, origX - padding),
          y: Math.min(height, origY + padding)
        }
      }
    })

    console.log('映射回原图的边界:', corners.value)
    drawCanvas()
    console.log('绘制完成')
  } else {
    console.log('未识别到区域')
  }
}

// 重置裁剪
const resetCrop = () => {
  initCropCorners()
  drawCanvas()
}

// 应用裁剪
const applyCrop = () => {
  if (!canvasRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const img = currentImage.value

  // 计算目标矩形的尺寸
  const p0 = corners.value[0]
  const p1 = corners.value[1]
  const p2 = corners.value[2]
  const p3 = corners.value[3]

  const topWidth = Math.sqrt((p1.x - p0.x) ** 2 + (p1.y - p0.y) ** 2)
  const bottomWidth = Math.sqrt((p2.x - p3.x) ** 2 + (p2.y - p3.y) ** 2)
  const width = Math.round((topWidth + bottomWidth) / 2)

  const leftHeight = Math.sqrt((p3.x - p0.x) ** 2 + (p3.y - p0.y) ** 2)
  const rightHeight = Math.sqrt((p2.x - p1.x) ** 2 + (p2.y - p1.y) ** 2)
  const height = Math.round((leftHeight + rightHeight) / 2)

  // 创建临时 Canvas
  const tempCanvas = document.createElement('canvas')
  tempCanvas.width = canvas.width
  tempCanvas.height = canvas.height
  const tempCtx = tempCanvas.getContext('2d')
  if (!tempCtx) return
  tempCtx.drawImage(img, 0, 0, canvas.width, canvas.height)
  const srcImageData = tempCtx.getImageData(0, 0, canvas.width, canvas.height)

  // 创建结果 Canvas
  const resultCanvas = document.createElement('canvas')
  resultCanvas.width = width
  resultCanvas.height = height
  const resultCtx = resultCanvas.getContext('2d')
  if (!resultCtx) return
  const resultImageData = resultCtx.createImageData(width, height)

  // 计算逆透视变换矩阵
  const dst = [
    { x: 0, y: 0 },
    { x: width, y: 0 },
    { x: width, y: height },
    { x: 0, y: height }
  ]

  const invMatrix = computePerspectiveTransform(dst, corners.value)

  // 逐像素映射
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const srcPoint = applyPerspectiveTransform({ x, y }, invMatrix)

      if (
        srcPoint.x >= 0 &&
        srcPoint.x < canvas.width &&
        srcPoint.y >= 0 &&
        srcPoint.y < canvas.height
      ) {
        const [r, g, b, a] = bilinearInterpolation(
          srcImageData,
          srcPoint.x,
          srcPoint.y
        )

        const destIdx = (y * width + x) * 4
        resultImageData.data[destIdx] = r
        resultImageData.data[destIdx + 1] = g
        resultImageData.data[destIdx + 2] = b
        resultImageData.data[destIdx + 3] = a
      }
    }
  }

  resultCtx.putImageData(resultImageData, 0, 0)

  // 更新当前图像
  const newImageData = resultCanvas.toDataURL('image/jpeg', 0.9)
  addToHistory(newImageData)
  return newImageData
}

// 预览旋转
const previewRotation = () => {
  if (!canvasRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const angle = (rotationAngle.value * Math.PI) / 180

  // 清空并旋转
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.save()
  ctx.translate(canvas.width / 2, canvas.height / 2)
  ctx.rotate(angle)
  ctx.translate(-canvas.width / 2, -canvas.height / 2)
  ctx.drawImage(currentImage.value, 0, 0, canvas.width, canvas.height)
  ctx.restore()
}

// 应用旋转
const rotate = (angle: number) => {
  rotationAngle.value = (rotationAngle.value + angle) % 360
  applyRotation()
}

const applyRotation = () => {
  if (!canvasRef.value || !currentImage.value) return

  const img = currentImage.value

  const angle = (rotationAngle.value * Math.PI) / 180

  // 计算旋转后的尺寸
  const cos = Math.abs(Math.cos(angle))
  const sin = Math.abs(Math.sin(angle))
  const newWidth = img.width * cos + img.height * sin
  const newHeight = img.width * sin + img.height * cos

  // 创建新 Canvas
  const resultCanvas = document.createElement('canvas')
  resultCanvas.width = newWidth
  resultCanvas.height = newHeight
  const resultCtx = resultCanvas.getContext('2d')
  if (!resultCtx) return

  // 旋转图像
  resultCtx.translate(newWidth / 2, newHeight / 2)
  resultCtx.rotate(angle)
  resultCtx.drawImage(img, -img.width / 2, -img.height / 2)

  const newImageData = resultCanvas.toDataURL('image/jpeg', 0.9)
  addToHistory(newImageData)
  rotationAngle.value = 0
}

// 预览对比度
const previewContrast = () => {
  if (!canvasRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.drawImage(currentImage.value, 0, 0, canvas.width, canvas.height)

  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
  const data = imageData.data

  const contrastFactor = (contrast.value - 100) / 100
  const brightnessFactor = ((brightness.value - 100) / 100) * 255

  for (let i = 0; i < data.length; i += 4) {
    // 应用对比度
    data[i] = (data[i] - 128) * (1 + contrastFactor) + 128 + brightnessFactor
    data[i + 1] =
      (data[i + 1] - 128) * (1 + contrastFactor) + 128 + brightnessFactor
    data[i + 2] =
      (data[i + 2] - 128) * (1 + contrastFactor) + 128 + brightnessFactor
  }

  ctx.putImageData(imageData, 0, 0)
}

// 重置对比度
const resetContrast = () => {
  contrast.value = 100
  brightness.value = 100
  previewContrast()
}

// 应用对比度
const applyContrast = () => {
  if (!canvasRef.value) return

  const newImageData = canvasRef.value.toDataURL('image/jpeg', 0.9)
  addToHistory(newImageData)

  contrast.value = 100
  brightness.value = 100
}

// 预览二值化
const previewThreshold = () => {
  if (!canvasRef.value || !currentImage.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.drawImage(currentImage.value, 0, 0, canvas.width, canvas.height)

  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
  const data = imageData.data

  for (let i = 0; i < data.length; i += 4) {
    // 转换为灰度
    const gray = data[i] * 0.299 + data[i + 1] * 0.587 + data[i + 2] * 0.114

    // 二值化
    const value = gray >= threshold.value ? 255 : 0

    if (invertThreshold.value) {
      data[i] = 255 - value
      data[i + 1] = 255 - value
      data[i + 2] = 255 - value
    } else {
      data[i] = value
      data[i + 1] = value
      data[i + 2] = value
    }
  }

  ctx.putImageData(imageData, 0, 0)
}

// 重置二值化
const resetThreshold = () => {
  threshold.value = 128
  invertThreshold.value = false
  previewThreshold()
}

// 应用二值化
const applyThreshold = () => {
  if (!canvasRef.value) return

  const newImageData = canvasRef.value.toDataURL('image/jpeg', 0.9)
  addToHistory(newImageData)

  threshold.value = 128
  invertThreshold.value = false
}

// 添加到历史记录
const addToHistory = (imageData: string) => {
  // 删除当前位置之后的历史
  history.value = history.value.slice(0, historyIndex.value + 1)

  // 添加新状态
  history.value.push(imageData)
  historyIndex.value = history.value.length - 1

  // 加载新图像
  loadHistoryImage()
}

// 加载历史图像
const loadHistoryImage = () => {
  const img = new Image()
  img.onload = () => {
    currentImage.value = img

    if (!canvasRef.value) return

    const canvas = canvasRef.value
    canvas.width = img.width
    canvas.height = img.height

    initCropCorners()
    drawCanvas()
  }
  img.src = history.value[historyIndex.value]
}

// 撤销
const undo = () => {
  if (historyIndex.value > 0) {
    historyIndex.value--
    loadHistoryImage()
  }
}

// 重做
const redo = () => {
  if (historyIndex.value < history.value.length - 1) {
    historyIndex.value++
    loadHistoryImage()
  }
}

// 重置所有参数
const resetAllParameters = () => {
  rotationAngle.value = 0
  contrast.value = 100
  brightness.value = 100
  threshold.value = 128
  invertThreshold.value = false
}

// 重置所有
const resetAll = () => {
  if (!originalImage.value) return

  history.value = [props.imageData]
  historyIndex.value = 0

  currentImage.value = originalImage.value
  resetAllParameters()

  if (!canvasRef.value) return

  const canvas = canvasRef.value
  canvas.width = originalImage.value.width
  canvas.height = originalImage.value.height

  initCropCorners()
  drawCanvas()
}

// 透视变换相关函数
const computePerspectiveTransform = (
  src: { x: number; y: number }[],
  dst: { x: number; y: number }[]
) => {
  const A: number[][] = []
  const b: number[] = []

  for (let i = 0; i < 4; i++) {
    const sx = src[i].x
    const sy = src[i].y
    const dx = dst[i].x
    const dy = dst[i].y

    A.push([sx, sy, 1, 0, 0, 0, -sx * dx, -sy * dx])
    A.push([0, 0, 0, sx, sy, 1, -sx * dy, -sy * dy])
    b.push(dx)
    b.push(dy)
  }

  const n = 8
  for (let i = 0; i < n; i++) {
    let maxRow = i
    for (let k = i + 1; k < n; k++) {
      if (Math.abs(A[k][i]) > Math.abs(A[maxRow][i])) {
        maxRow = k
      }
    }

    ;[A[i], A[maxRow]] = [A[maxRow], A[i]]
    ;[b[i], b[maxRow]] = [b[maxRow], b[i]]

    for (let k = i + 1; k < n; k++) {
      const factor = A[k][i] / A[i][i]
      for (let j = i; j < n; j++) {
        A[k][j] -= factor * A[i][j]
      }
      b[k] -= factor * b[i]
    }
  }

  const x = new Array(n).fill(0)
  for (let i = n - 1; i >= 0; i--) {
    x[i] = b[i]
    for (let j = i + 1; j < n; j++) {
      x[i] -= A[i][j] * x[j]
    }
    x[i] /= A[i][i]
  }

  return [
    [x[0], x[1], x[2]],
    [x[3], x[4], x[5]],
    [x[6], x[7], 1]
  ]
}

const applyPerspectiveTransform = (
  point: { x: number; y: number },
  matrix: number[][]
) => {
  const x = point.x
  const y = point.y

  const w = matrix[2][0] * x + matrix[2][1] * y + matrix[2][2]
  const newX = (matrix[0][0] * x + matrix[0][1] * y + matrix[0][2]) / w
  const newY = (matrix[1][0] * x + matrix[1][1] * y + matrix[1][2]) / w

  return { x: newX, y: newY }
}

const bilinearInterpolation = (imageData: ImageData, x: number, y: number) => {
  const width = imageData.width
  const height = imageData.height
  const data = imageData.data

  const x1 = Math.floor(x)
  const y1 = Math.floor(y)
  const x2 = Math.min(x1 + 1, width - 1)
  const y2 = Math.min(y1 + 1, height - 1)

  const dx = x - x1
  const dy = y - y1

  const idx1 = (y1 * width + x1) * 4
  const idx2 = (y1 * width + x2) * 4
  const idx3 = (y2 * width + x1) * 4
  const idx4 = (y2 * width + x2) * 4

  const r =
    (1 - dx) * (1 - dy) * data[idx1] +
    dx * (1 - dy) * data[idx2] +
    (1 - dx) * dy * data[idx3] +
    dx * dy * data[idx4]

  const g =
    (1 - dx) * (1 - dy) * data[idx1 + 1] +
    dx * (1 - dy) * data[idx2 + 1] +
    (1 - dx) * dy * data[idx3 + 1] +
    dx * dy * data[idx4 + 1]

  const b =
    (1 - dx) * (1 - dy) * data[idx1 + 2] +
    dx * (1 - dy) * data[idx2 + 2] +
    (1 - dx) * dy * data[idx3 + 2] +
    dx * dy * data[idx4 + 2]

  const a =
    (1 - dx) * (1 - dy) * data[idx1 + 3] +
    dx * (1 - dy) * data[idx2 + 3] +
    (1 - dx) * dy * data[idx3 + 3] +
    dx * dy * data[idx4 + 3]

  return [r, g, b, a]
}

// 确认编辑
const handleConfirm = () => {
  if (!canvasRef.value) return

  let imageData = ''

  // 如果当前工具是裁剪，先按框选区域裁剪再确认
  if (currentTool.value === 'crop') {
    const cropped = applyCrop()
    imageData = cropped || canvasRef.value.toDataURL('image/jpeg', 1.0)
  } else {
    imageData = canvasRef.value.toDataURL('image/jpeg', 1.0)
  }
  emit('confirm', imageData)
}

// 取消编辑
const handleCancel = () => {
  emit('close')
}
</script>

<style scoped>
.edit-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #000;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  animation: editIn 0.2s ease;
}

@keyframes editIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.edit-header {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 16px 20px;
  padding-top: calc(16px + env(safe-area-inset-top));
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.6), transparent);
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}

.header-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.25s ease;
  backdrop-filter: blur(8px);
}

.header-btn:hover {
  background: rgba(255, 255, 255, 0.25);
  transform: scale(1.05);
}

.header-btn:active {
  transform: scale(0.92);
}

.confirm-btn {
  background: var(--primary-color, #1976d2) !important;
  border-color: var(--primary-color, #1976d2) !important;
  font-size: 20px;
  font-weight: 700;
}

.confirm-btn:hover {
  opacity: 0.85;
}

.edit-title {
  color: #fff;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.toolbar {
  position: absolute;
  top: calc(56px + env(safe-area-inset-top));
  left: 0;
  right: 0;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(12px);
  display: flex;
  gap: 8px;
  align-items: center;
  z-index: 9;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}

.toolbar::-webkit-scrollbar {
  display: none;
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background: rgba(255, 255, 255, 0.15);
  margin: 0 4px;
  flex-shrink: 0;
}

.tool-btn {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.8);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
  transform: translateY(-1px);
}

.tool-btn.active {
  background: var(--primary-color, #1976d2);
  color: #fff;
  border-color: var(--primary-color, #1976d2);
  box-shadow: 0 0 16px rgba(25, 118, 210, 0.4);
}

.tool-btn:disabled {
  opacity: 0.2;
  cursor: not-allowed;
  transform: none !important;
}

.adjustment-panel {
  position: absolute;
  top: calc(120px + env(safe-area-inset-top));
  left: 16px;
  right: 16px;
  background: rgba(20, 20, 30, 0.95);
  border-radius: 14px;
  padding: 18px;
  z-index: 8;
  max-height: 55vh;
  overflow-y: auto;
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.5);
  animation: panelSlide 0.2s ease;
}

@keyframes panelSlide {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.panel-header span {
  color: #fff;
  font-size: 16px;
  font-weight: 600;
}

.panel-close {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  border: none;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.panel-close:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.panel-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hint {
  color: rgba(255, 255, 255, 0.6);
  font-size: 13px;
  text-align: center;
  margin: 0;
  line-height: 1.5;
}

.crop-buttons,
.rotate-controls,
.contrast-controls,
.threshold-controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.control-btn {
  padding: 10px 16px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.85);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.14);
  color: #fff;
}

.control-btn.primary {
  background: var(--primary-color, #1976d2);
  border-color: var(--primary-color, #1976d2);
  color: #fff;
}

.control-btn.primary:hover {
  opacity: 0.85;
}

.slider-control {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.slider-control label {
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
}

.slider-control input[type='range'] {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.15);
  outline: none;
  -webkit-appearance: none;
  appearance: none;
}

.slider-control input[type='range']::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--primary-color, #1976d2);
  border: 2px solid #fff;
  cursor: pointer;
  box-shadow: 0 1px 6px rgba(0, 0, 0, 0.3);
}

.checkbox-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.checkbox-control input[type='checkbox'] {
  width: 18px;
  height: 18px;
  accent-color: var(--primary-color, #1976d2);
  cursor: pointer;
}

.checkbox-control label {
  color: rgba(255, 255, 255, 0.8);
  font-size: 14px;
  cursor: pointer;
}

.edit-canvas-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  overflow: hidden;
  padding: 20px;
  padding-top: 120px;
  padding-bottom: 60px;
}

.edit-canvas-container canvas {
  max-width: 100%;
  max-height: 100%;
  touch-action: none;
  transition: transform 0.1s;
}

.status-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 12px 16px;
  padding-bottom: calc(12px + env(safe-area-inset-bottom));
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}

.status-bar span {
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
}

/* 放大镜 */
.loupe {
  position: fixed;
  width: 150px;
  height: 150px;
  border-radius: 50%;
  border: 3px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  pointer-events: none;
  z-index: 10000;
  background: #fff;
}

.loupe canvas {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
