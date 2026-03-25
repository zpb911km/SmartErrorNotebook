<template>
  <div class="edit-modal" v-if="visible">
    <div class="edit-header">
      <button class="header-btn close-btn" @click="handleCancel">✕</button>
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
      
      <button class="tool-btn" @click="undo" :disabled="historyIndex <= 0" title="撤销">
        ↩️
      </button>
      <button class="tool-btn" @click="redo" :disabled="historyIndex >= history.length - 1" title="重做">
        ↪️
      </button>
      
      <div class="toolbar-divider"></div>
      
      <button class="tool-btn" @click="resetAll" title="重置">
        🔄
      </button>
    </div>
    
    <!-- 调整面板 -->
    <div class="adjustment-panel" v-if="showAdjustmentPanel">
      <div class="panel-header">
        <span>{{ currentToolName }}</span>
        <button class="panel-close" @click="showAdjustmentPanel = false">✕</button>
      </div>
      
      <div class="panel-content">
        <!-- 裁剪工具 -->
        <div v-if="currentTool === 'crop'" class="crop-controls">
          <p class="hint">拖动四个角点框选裁剪区域</p>
          <div class="crop-buttons">
            <button class="control-btn" @click="resetCrop">重置选区</button>
            <button class="control-btn primary" @click="applyCrop">应用裁剪</button>
          </div>
        </div>
        
        <!-- 旋转工具 -->
        <div v-if="currentTool === 'rotate'" class="rotate-controls">
          <button class="control-btn" @click="rotate(-90)">↺ 左转90°</button>
          <button class="control-btn" @click="rotate(90)">↻ 右转90°</button>
          <button class="control-btn" @click="rotate(180)">↔ 180°</button>
          <div class="slider-control">
            <label>自由旋转: {{ rotationAngle }}°</label>
            <input type="range" v-model.number="rotationAngle" min="-180" max="180" @input="previewRotation">
          </div>
          <button class="control-btn primary" @click="applyRotation">应用旋转</button>
        </div>
        
        <!-- 对比度工具 -->
        <div v-if="currentTool === 'contrast'" class="contrast-controls">
          <div class="slider-control">
            <label>对比度: {{ contrast }}%</label>
            <input type="range" v-model.number="contrast" min="0" max="200" @input="previewContrast">
          </div>
          <div class="slider-control">
            <label>亮度: {{ brightness }}%</label>
            <input type="range" v-model.number="brightness" min="0" max="200" @input="previewContrast">
          </div>
          <button class="control-btn" @click="resetContrast">重置</button>
          <button class="control-btn primary" @click="applyContrast">应用</button>
        </div>
        
        <!-- 二值化工具 -->
        <div v-if="currentTool === 'threshold'" class="threshold-controls">
          <div class="slider-control">
            <label>阈值: {{ threshold }}</label>
            <input type="range" v-model.number="threshold" min="0" max="255" @input="previewThreshold">
          </div>
          <div class="checkbox-control">
            <input type="checkbox" id="invert" v-model="invertThreshold">
            <label for="invert">反色</label>
          </div>
          <button class="control-btn" @click="resetThreshold">重置</button>
          <button class="control-btn primary" @click="applyThreshold">应用</button>
        </div>
      </div>
    </div>
    
    <div class="edit-canvas-container" ref="containerRef">
      <canvas 
        ref="canvasRef"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
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
const corners = ref<{x: number; y: number}[]>([
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 }
])
const draggingCorner = ref<number | null>(null)
const pointRadius = 20

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

// 计算属性
const currentToolName = computed(() => {
  const tool = tools.find(t => t.id === currentTool.value)
  return tool ? tool.name : ''
})

const imageInfo = computed(() => {
  if (!currentImage.value) return ''
  return `${currentImage.value.width} × ${currentImage.value.height}px`
})

const cropInfo = computed(() => {
  if (currentTool.value !== 'crop' || corners.value.every(c => c.x === 0 && c.y === 0)) {
    return ''
  }
  const minX = Math.min(...corners.value.map(c => c.x))
  const minY = Math.min(...corners.value.map(c => c.y))
  const maxX = Math.max(...corners.value.map(c => c.x))
  const maxY = Math.max(...corners.value.map(c => c.y))
  return `选区: ${Math.round(maxX - minX)} × ${Math.round(maxY - minY)}px`
})

// 监听 visible 变化
watch(() => props.visible, async (newVal) => {
  console.log('ImageEditor visible changed:', newVal, 'autoDetect:', props.autoDetect)
  if (newVal && props.imageData) {
    await loadImage()
  }
})

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
  const padding = 50
  
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
const drawCropOverlay = (ctx: CanvasRenderingContext2D, canvas: HTMLCanvasElement) => {
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
  
  // 绘制四边形边框
  ctx.strokeStyle = '#1976d2'
  ctx.lineWidth = 3
  ctx.beginPath()
  ctx.moveTo(corners.value[0].x, corners.value[0].y)
  ctx.lineTo(corners.value[1].x, corners.value[1].y)
  ctx.lineTo(corners.value[2].x, corners.value[2].y)
  ctx.lineTo(corners.value[3].x, corners.value[3].y)
  ctx.closePath()
  ctx.stroke()
  
  // 绘制顶点
  corners.value.forEach((corner) => {
    ctx.beginPath()
    ctx.arc(corner.x, corner.y, pointRadius, 0, Math.PI * 2)
    ctx.fillStyle = '#1976d2'
    ctx.fill()
    ctx.strokeStyle = 'white'
    ctx.lineWidth = 2
    ctx.stroke()
  })
}

// 获取 Canvas 的缩放比例
const getCanvasScale = () => {
  if (!canvasRef.value) return { scaleX: 1, scaleY: 1 }
  
  const rect = canvasRef.value.getBoundingClientRect()
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

// 全局鼠标事件处理函数
const handleGlobalMouseMove = (e: MouseEvent) => {
  if (currentTool.value === 'crop' && draggingCorner.value !== null) {
    const { x, y } = screenToCanvas(e.clientX, e.clientY)
    
    // 限制调节点在图片范围内
    if (canvasRef.value) {
      const width = canvasRef.value.width
      const height = canvasRef.value.height
      
      const clampedX = Math.max(0, Math.min(x, width))
      const clampedY = Math.max(0, Math.min(y, height))
      
      corners.value[draggingCorner.value] = { x: clampedX, y: clampedY }
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
  // 移除全局事件监听器
  window.removeEventListener('mousemove', handleGlobalMouseMove)
  window.removeEventListener('mouseup', handleGlobalMouseUp)
  
  // 限制调节点在图片范围内
  if (canvasRef.value) {
    const width = canvasRef.value.width
    const height = canvasRef.value.height
    
    corners.value = corners.value.map((corner) => {
      return {
        x: Math.max(0, Math.min(corner.x, width)),
        y: Math.max(0, Math.min(corner.y, height))
      }
    })
    
    drawCanvas()
  }
  
  draggingCorner.value = null
  isPanning.value = false
}

// 鼠标事件处理
const handleMouseDown = (e: MouseEvent) => {
  if (currentTool.value === 'crop') {
    const { x, y } = screenToCanvas(e.clientX, e.clientY)
    
    corners.value.forEach((corner, index) => {
      const distance = Math.sqrt((x - corner.x) ** 2 + (y - corner.y) ** 2)
      if (distance <= pointRadius) {
        draggingCorner.value = index
        // 添加全局事件监听器
        window.addEventListener('mousemove', handleGlobalMouseMove)
        window.addEventListener('mouseup', handleGlobalMouseUp)
      }
    })
  } else {
    isPanning.value = true
    lastPanPos.value = { x: e.clientX, y: e.clientY }
    // 添加全局事件监听器
    window.addEventListener('mousemove', handleGlobalMouseMove)
    window.addEventListener('mouseup', handleGlobalMouseUp)
  }
}

const handleMouseMove = (e: MouseEvent) => {
  // 这个函数现在可以为空，因为我们使用全局事件监听器
}

const handleMouseUp = () => {
  // 这个函数现在可以为空，因为我们使用全局事件监听器
}

// 触摸事件处理
const handleTouchStart = (e: TouchEvent) => {
  e.preventDefault()
  if (currentTool.value === 'crop') {
    const touch = e.touches[0]
    const { x, y } = screenToCanvas(touch.clientX, touch.clientY)
    
    corners.value.forEach((corner, index) => {
      const distance = Math.sqrt((x - corner.x) ** 2 + (y - corner.y) ** 2)
      if (distance <= pointRadius) {
        draggingCorner.value = index
      }
    })
  }
}

const handleTouchMove = (e: TouchEvent) => {
  e.preventDefault()
  if (currentTool.value === 'crop' && draggingCorner.value !== null) {
    const touch = e.touches[0]
    const { x, y } = screenToCanvas(touch.clientX, touch.clientY)
    
    // 限制调节点在图片范围内
    if (canvasRef.value) {
      const width = canvasRef.value.width
      const height = canvasRef.value.height
      
      const clampedX = Math.max(0, Math.min(x, width))
      const clampedY = Math.max(0, Math.min(y, height))
      
      corners.value[draggingCorner.value] = { x: clampedX, y: clampedY }
      drawCanvas()
    }
  }
}

const handleTouchEnd = () => {
  draggingCorner.value = null
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
const applyThresholdToImage = (imageData: ImageData, threshold: number): ImageData => {
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
const floodFill = (x: number, y: number, width: number, height: number, data: Uint8ClampedArray, visited: boolean[]): {x: number, y: number}[] => {
  const stack = [{x, y}]
  const contour: {x: number, y: number}[] = []
  const directions = [[0, 1], [1, 0], [0, -1], [-1, 0]]
  
  while (stack.length > 0) {
    const {x, y} = stack.pop()!
    const idx = y * width + x
    
    if (x < 0 || x >= width || y < 0 || y >= height || visited[idx] || data[idx * 4] <= 128) {
      continue
    }
    
    visited[idx] = true
    contour.push({x, y})
    
    for (const [dx, dy] of directions) {
      stack.push({x: x + dx, y: y + dy})
    }
  }
  
  return contour
}

// 提取轮廓
const findContours = (imageData: ImageData): {x: number, y: number}[] => {
  const data = imageData.data
  const width = imageData.width
  const height = imageData.height
  const visited = new Array(width * height).fill(false)
  const contours: {x: number, y: number}[] = []
  
  // 简单的轮廓提取算法
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4
      if (data[idx] > 128 && !visited[y * width + x]) {
        // 找到一个轮廓
        const contour = floodFill(x, y, width, height, data, visited)
        if (contour.length > 100) { // 过滤小轮廓
          contours.push(...contour)
        }
      }
    }
  }
  
  return contours
}

// 拟合矩形
const fitRectangle = (points: {x: number, y: number}[]): {x: number, y: number}[] => {
  if (points.length === 0) return []
  
  const minX = Math.min(...points.map(p => p.x))
  const minY = Math.min(...points.map(p => p.y))
  const maxX = Math.max(...points.map(p => p.x))
  const maxY = Math.max(...points.map(p => p.y))
  
  return [
    {x: minX, y: minY},
    {x: maxX, y: minY},
    {x: maxX, y: maxY},
    {x: minX, y: maxY}
  ]
}

// 自动识别区域
const autoDetectRegion = () => {
  console.log('自动识别开始')
  if (!canvasRef.value || !currentImage.value) {
    console.log('Canvas或图像不存在')
    return
  }
  
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    console.log('无法获取Canvas上下文')
    return
  }
  
  console.log('Canvas尺寸:', canvas.width, 'x', canvas.height)
  
  // 绘制当前图像
  ctx.drawImage(currentImage.value, 0, 0, canvas.width, canvas.height)
  
  // 获取图像数据
  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
  console.log('获取图像数据成功')
  
  // 应用二值化增强对比度
  const thresholdedData = applyThresholdToImage(imageData, 128)
  console.log('二值化处理完成')
  
  // 边缘检测
  const edgeData = sobelEdgeDetection(thresholdedData)
  console.log('边缘检测完成')
  
  // 提取轮廓
  const contours = findContours(edgeData)
  console.log('提取轮廓完成，轮廓点数量:', contours.length)
  
  // 拟合矩形
  const rectangle = fitRectangle(contours)
  console.log('拟合矩形完成，矩形点数量:', rectangle.length)
  
  if (rectangle.length === 4) {
    console.log('识别到区域:', rectangle)
    // 调整边界，添加一些 padding
    const padding = 20
    const width = canvas.width
    const height = canvas.height
    
    corners.value = rectangle.map((point, index) => {
      if (index === 0) { // 左上角
        return { x: Math.max(0, point.x - padding), y: Math.max(0, point.y - padding) }
      } else if (index === 1) { // 右上角
        return { x: Math.min(width, point.x + padding), y: Math.max(0, point.y - padding) }
      } else if (index === 2) { // 右下角
        return { x: Math.min(width, point.x + padding), y: Math.min(height, point.y + padding) }
      } else { // 左下角
        return { x: Math.max(0, point.x - padding), y: Math.min(height, point.y + padding) }
      }
    })
    
    console.log('调整后的边界:', corners.value)
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
      
      if (srcPoint.x >= 0 && srcPoint.x < canvas.width &&
          srcPoint.y >= 0 && srcPoint.y < canvas.height) {
        
        const [r, g, b, a] = bilinearInterpolation(srcImageData, srcPoint.x, srcPoint.y)
        
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
}

// 预览旋转
const previewRotation = () => {
  if (!canvasRef.value || !currentImage.value) return
  
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  
  const angle = rotationAngle.value * Math.PI / 180
  
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
  previewRotation()
}

const applyRotation = () => {
  if (!canvasRef.value || !currentImage.value) return
  
  const canvas = canvasRef.value
  const img = currentImage.value
  
  const angle = rotationAngle.value * Math.PI / 180
  
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
  const brightnessFactor = (brightness.value - 100) / 100 * 255
  
  for (let i = 0; i < data.length; i += 4) {
    // 应用对比度
    data[i] = ((data[i] - 128) * (1 + contrastFactor) + 128 + brightnessFactor)
    data[i + 1] = ((data[i + 1] - 128) * (1 + contrastFactor) + 128 + brightnessFactor)
    data[i + 2] = ((data[i + 2] - 128) * (1 + contrastFactor) + 128 + brightnessFactor)
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
const computePerspectiveTransform = (src: {x: number; y: number}[], dst: {x: number; y: number}[]) => {
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
    
    [A[i], A[maxRow]] = [A[maxRow], A[i]]
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

const applyPerspectiveTransform = (point: {x: number; y: number}, matrix: number[][]) => {
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
  
  const r = (1 - dx) * (1 - dy) * data[idx1] +
           dx * (1 - dy) * data[idx2] +
           (1 - dx) * dy * data[idx3] +
           dx * dy * data[idx4]
           
  const g = (1 - dx) * (1 - dy) * data[idx1 + 1] +
           dx * (1 - dy) * data[idx2 + 1] +
           (1 - dx) * dy * data[idx3 + 1] +
           dx * dy * data[idx4 + 1]
           
  const b = (1 - dx) * (1 - dy) * data[idx1 + 2] +
           dx * (1 - dy) * data[idx2 + 2] +
           (1 - dx) * dy * data[idx3 + 2] +
           dx * dy * data[idx4 + 2]
           
  const a = (1 - dx) * (1 - dy) * data[idx1 + 3] +
           dx * (1 - dy) * data[idx2 + 3] +
           (1 - dx) * dy * data[idx3 + 3] +
           dx * dy * data[idx4 + 3]
           
  return [r, g, b, a]
}

// 确认编辑
const handleConfirm = () => {
  if (!canvasRef.value) return
  const imageData = canvasRef.value.toDataURL('image/jpeg', 0.9)
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
}

.edit-header {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 16px 20px;
  padding-top: calc(16px + env(safe-area-inset-top));
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.5), transparent);
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}

.header-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.3s;
}

.header-btn:active {
  background: rgba(255, 255, 255, 0.3);
}

.edit-title {
  color: white;
  font-size: 16px;
  font-weight: 500;
}

.toolbar {
  position: absolute;
  top: calc(56px + env(safe-area-inset-top));
  left: 0;
  right: 0;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  gap: 8px;
  align-items: center;
  z-index: 9;
  overflow-x: auto;
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background: rgba(255, 255, 255, 0.3);
  margin: 0 4px;
}

.tool-btn {
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  width: 40px;
  height: 40px;
  border-radius: 8px;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s;
  flex-shrink: 0;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.tool-btn.active {
  background: var(--primary-color);
}

.tool-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.adjustment-panel {
  position: absolute;
  top: calc(120px + env(safe-area-inset-top));
  left: 16px;
  right: 16px;
  background: rgba(30, 30, 30, 0.95);
  border-radius: 12px;
  padding: 16px;
  z-index: 8;
  max-height: 50vh;
  overflow-y: auto;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.panel-header span {
  color: white;
  font-size: 16px;
  font-weight: 500;
}

.panel-close {
  background: transparent;
  border: none;
  color: white;
  font-size: 18px;
  cursor: pointer;
  padding: 4px 8px;
}

.panel-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hint {
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
  text-align: center;
  margin: 0;
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
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.control-btn.primary {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.slider-control {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.slider-control label {
  color: white;
  font-size: 14px;
}

.slider-control input[type="range"] {
  width: 100%;
  height: 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.2);
  outline: none;
  -webkit-appearance: none;
}

.slider-control input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--primary-color);
  cursor: pointer;
}

.checkbox-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.checkbox-control input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.checkbox-control label {
  color: white;
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
</style>