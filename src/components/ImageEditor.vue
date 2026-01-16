<template>
  <div class="edit-modal" v-if="visible">
    <div class="edit-header">
      <button class="header-btn close-btn" @click="handleCancel">✕</button>
      <span class="edit-title">框选题目区域</span>
      <button class="header-btn confirm-btn" @click="handleConfirm">✓</button>
    </div>
    
    <div class="edit-canvas-container" ref="containerRef">
      <canvas 
        ref="canvasRef"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseUp"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
      ></canvas>
    </div>
    
    <div class="edit-controls">
      <button class="control-btn" @click="resetCorners">重置选区</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'

interface Props {
  visible: boolean
  imageData: string
}

interface Emits {
  (e: 'close'): void
  (e: 'confirm', imageData: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const image = ref<HTMLImageElement | null>(null)
const corners = ref<{x: number; y: number}[]>([
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 }
])
const draggingCorner = ref<number | null>(null)
const pointRadius = 20

// 监听 visible 变化
watch(() => props.visible, async (newVal) => {
  if (newVal && props.imageData) {
    await loadImage()
  }
})

// 加载图片
const loadImage = () => {
  const img = new Image()
  img.onload = () => {
    image.value = img
    nextTick(() => {
      initCanvas(img)
    })
  }
  img.src = props.imageData
}

// 初始化 Canvas
const initCanvas = (img: HTMLImageElement) => {
  if (!canvasRef.value || !containerRef.value) return
  
  const canvas = canvasRef.value
  const container = containerRef.value
  
  // 设置 Canvas 尺寸
  const maxWidth = container.clientWidth
  const maxHeight = container.clientHeight
  
  const scale = Math.min(maxWidth / img.width, maxHeight / img.height, 1)
  canvas.width = img.width * scale
  canvas.height = img.height * scale
  
  // 初始化四个角点（留出边距）
  const padding = 50
  corners.value = [
    { x: padding, y: padding },
    { x: canvas.width - padding, y: padding },
    { x: canvas.width - padding, y: canvas.height - padding },
    { x: padding, y: canvas.height - padding }
  ]
  drawCanvas()
}

// 绘制 Canvas
const drawCanvas = () => {
  if (!canvasRef.value || !image.value) return
  
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  
  if (!ctx) return
  
  const img = image.value
  
  // 清空 Canvas
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  
  // 绘制图片
  ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
  
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
  ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
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
  corners.value.forEach((corner, index) => {
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

// 鼠标事件处理
const handleMouseDown = (e: MouseEvent) => {
  const rect = canvasRef.value?.getBoundingClientRect()
  if (!rect) return
  
  const { x, y } = screenToCanvas(e.clientX, e.clientY)
  
  // 检查是否点击了某个顶点
  corners.value.forEach((corner, index) => {
    const distance = Math.sqrt((x - corner.x) ** 2 + (y - corner.y) ** 2)
    if (distance <= pointRadius) {
      draggingCorner.value = index
    }
  })
}

const handleMouseMove = (e: MouseEvent) => {
  if (draggingCorner.value === null) return
  
  const { x, y } = screenToCanvas(e.clientX, e.clientY)
  
  // 限制在 Canvas 范围内
  const clampedX = Math.max(0, Math.min(x, canvasRef.value!.width))
  const clampedY = Math.max(0, Math.min(y, canvasRef.value!.height))
  
  corners.value[draggingCorner.value] = { x: clampedX, y: clampedY }
  drawCanvas()
}

const handleMouseUp = () => {
  draggingCorner.value = null
}

// 触摸事件处理
const handleTouchStart = (e: TouchEvent) => {
  e.preventDefault()
  const rect = canvasRef.value?.getBoundingClientRect()
  if (!rect) return
  
  const touch = e.touches[0]
  const { x, y } = screenToCanvas(touch.clientX, touch.clientY)
  
  corners.value.forEach((corner, index) => {
    const distance = Math.sqrt((x - corner.x) ** 2 + (y - corner.y) ** 2)
    if (distance <= pointRadius) {
      draggingCorner.value = index
    }
  })
}

const handleTouchMove = (e: TouchEvent) => {
  e.preventDefault()
  if (draggingCorner.value === null) return
  
  const touch = e.touches[0]
  const { x, y } = screenToCanvas(touch.clientX, touch.clientY)
  
  // 限制在 Canvas 范围内
  const clampedX = Math.max(0, Math.min(x, canvasRef.value!.width))
  const clampedY = Math.max(0, Math.min(y, canvasRef.value!.height))
  
  corners.value[draggingCorner.value] = { x: clampedX, y: clampedY }
  drawCanvas()
}

const handleTouchEnd = () => {
  draggingCorner.value = null
}

// 重置选区
const resetCorners = () => {
  if (!canvasRef.value || !image.value) return
  
  const canvas = canvasRef.value
  const padding = 50
  corners.value = [
    { x: padding, y: padding },
    { x: canvas.width - padding, y: padding },
    { x: canvas.width - padding, y: canvas.height - padding },
    { x: padding, y: canvas.height - padding }
  ]
  drawCanvas()
}

// 计算透视变换矩阵
const computePerspectiveTransform = (src: {x: number; y: number}[], dst: {x: number; y: number}[]) => {
  // 求解透视变换矩阵 H，使得 H * src = dst
  // 使用高斯消元法求解线性方程组
  
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
  
  // 高斯消元法求解
  const n = 8
  for (let i = 0; i < n; i++) {
    // 找到主元
    let maxRow = i
    for (let k = i + 1; k < n; k++) {
      if (Math.abs(A[k][i]) > Math.abs(A[maxRow][i])) {
        maxRow = k
      }
    }
    
    // 交换行
    [A[i], A[maxRow]] = [A[maxRow], A[i]]
    ;[b[i], b[maxRow]] = [b[maxRow], b[i]]
    
    // 消元
    for (let k = i + 1; k < n; k++) {
      const factor = A[k][i] / A[i][i]
      for (let j = i; j < n; j++) {
        A[k][j] -= factor * A[i][j]
      }
      b[k] -= factor * b[i]
    }
  }
  
  // 回代
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

// 应用透视变换矩阵到点
const applyPerspectiveTransform = (point: {x: number; y: number}, matrix: number[][]) => {
  const x = point.x
  const y = point.y
  
  const w = matrix[2][0] * x + matrix[2][1] * y + matrix[2][2]
  const newX = (matrix[0][0] * x + matrix[0][1] * y + matrix[0][2]) / w
  const newY = (matrix[1][0] * x + matrix[1][1] * y + matrix[1][2]) / w
  
  return { x: newX, y: newY }
}

// 双线性插值
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

// 透视变换
const perspectiveTransform = () => {
  if (!canvasRef.value || !image.value) return null
  
  const canvas = canvasRef.value
  const img = image.value
  
  // 计算目标矩形的尺寸（基于四边形的边长）
  const p0 = corners.value[0]
  const p1 = corners.value[1]
  const p2 = corners.value[2]
  const p3 = corners.value[3]
  
  // 计算四边形的宽度（取上下边的平均值）
  const topWidth = Math.sqrt((p1.x - p0.x) ** 2 + (p1.y - p0.y) ** 2)
  const bottomWidth = Math.sqrt((p2.x - p3.x) ** 2 + (p2.y - p3.y) ** 2)
  const width = Math.round((topWidth + bottomWidth) / 2)
  
  // 计算四边形的高度（取左右边的平均值）
  const leftHeight = Math.sqrt((p3.x - p0.x) ** 2 + (p3.y - p0.y) ** 2)
  const rightHeight = Math.sqrt((p2.x - p1.x) ** 2 + (p2.y - p1.y) ** 2)
  const height = Math.round((leftHeight + rightHeight) / 2)
  
  // 创建临时 Canvas 获取原始图像数据
  const tempCanvas = document.createElement('canvas')
  tempCanvas.width = canvas.width
  tempCanvas.height = canvas.height
  const tempCtx = tempCanvas.getContext('2d')
  if (!tempCtx) return null
  tempCtx.drawImage(img, 0, 0, canvas.width, canvas.height)
  const srcImageData = tempCtx.getImageData(0, 0, canvas.width, canvas.height)
  
  // 创建结果 Canvas
  const resultCanvas = document.createElement('canvas')
  resultCanvas.width = width
  resultCanvas.height = height
  const resultCtx = resultCanvas.getContext('2d')
  if (!resultCtx) return null
  const resultImageData = resultCtx.createImageData(width, height)
  
  // 计算从目标矩形到源四边形的逆透视变换矩阵
  const dst = [
    { x: 0, y: 0 },
    { x: width, y: 0 },
    { x: width, y: height },
    { x: 0, y: height }
  ]
  
  // 计算从目标到源的变换矩阵（逆变换）
  const invMatrix = computePerspectiveTransform(dst, corners.value)
  
  // 对目标图像的每个像素进行逆变换
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      // 通过逆变换找到源图像中的对应位置
      const srcPoint = applyPerspectiveTransform({ x, y }, invMatrix)
      
      // 检查是否在源图像范围内
      if (srcPoint.x >= 0 && srcPoint.x < canvas.width &&
          srcPoint.y >= 0 && srcPoint.y < canvas.height) {
        
        // 使用双线性插值获取像素值
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
  
  return resultCanvas.toDataURL('image/jpeg', 0.9)
}

// 确认编辑
const handleConfirm = () => {
  const transformedImage = perspectiveTransform()
  if (transformedImage) {
    emit('confirm', transformedImage)
  } else {
    // 如果透视变换失败，使用原图
    emit('confirm', props.imageData)
  }
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

.edit-canvas-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  overflow: hidden;
  padding: 20px;
}

.edit-canvas-container canvas {
  max-width: 100%;
  max-height: 100%;
  touch-action: none;
}

.edit-controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 20px;
  padding-bottom: calc(20px + env(safe-area-inset-bottom));
  background: linear-gradient(to top, rgba(0, 0, 0, 0.5), transparent);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10;
}

.control-btn {
  padding: 12px 24px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.control-btn:active {
  background: rgba(255, 255, 255, 0.2);
}
</style>