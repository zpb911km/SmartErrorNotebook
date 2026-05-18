<template>
  <div class="camera-modal" v-if="visible">
    <div class="camera-container">
      <div class="camera-header">
        <button class="header-btn close-btn" @click="handleClose">✕</button>
        <span class="camera-title">拍照</span>
        <button
          class="header-btn switch-btn"
          @click="handleSwitchCamera"
          v-if="hasMultipleCameras"
        >
          🔄
        </button>
      </div>

      <div class="camera-preview">
        <video ref="videoRef" autoplay playsinline></video>
        <canvas ref="canvasRef" style="display: none"></canvas>

        <!-- 错误提示 -->
        <div class="camera-error" v-if="error">
          <div class="error-icon">⚠️</div>
          <p>{{ error }}</p>
          <button class="retry-btn" @click="startCamera">重试</button>
        </div>
      </div>

      <div class="camera-controls">
        <button
          class="capture-btn"
          @click="handleCapture"
          :disabled="error.length > 0"
        >
          <div class="capture-inner"></div>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount, watch } from 'vue'

interface Props {
  visible: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'error'): void
  (e: 'capture', imageData: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const videoRef = ref<HTMLVideoElement | null>(null)
const canvasRef = ref<HTMLCanvasElement | null>(null)
const error = ref('')
const mediaStream = ref<MediaStream | null>(null)
const currentCamera = ref<'user' | 'environment'>('environment')
const hasMultipleCameras = ref(false)

// 监听 visible 变化
watch(
  () => props.visible,
  async (newVal) => {
    if (newVal) {
      await startCamera()
    } else {
      stopCamera()
    }
  }
)

// 启动相机
const startCamera = async () => {
  error.value = ''

  try {
    // 停止之前的流
    if (mediaStream.value) {
      mediaStream.value.getTracks().forEach((track) => track.stop())
    }

    // 请求摄像头权限
    const constraints: MediaStreamConstraints = {
      video: {
        facingMode: currentCamera.value,
        width: { ideal: 1920 },
        height: { ideal: 1080 }
      },
      audio: false
    }

    const stream = await navigator.mediaDevices.getUserMedia(constraints)
    mediaStream.value = stream

    // 设置视频源
    if (videoRef.value) {
      videoRef.value.srcObject = stream
    }

    // 检查是否有多个摄像头
    const devices = await navigator.mediaDevices.enumerateDevices()
    const cameras = devices.filter((device) => device.kind === 'videoinput')
    hasMultipleCameras.value = cameras.length > 1
  } catch (err: any) {
    console.error('相机启动失败:', err)
    if (err.name === 'NotAllowedError') {
      error.value = '请允许访问摄像头权限'
    } else if (err.name === 'NotFoundError') {
      error.value = '未找到摄像头设备'
    } else if (err.name === 'NotReadableError') {
      error.value = '摄像头被其他应用占用'
    } else {
      error.value = '相机启动失败，请检查设备'
    }
    emit('error')
  }
}

// 停止相机
const stopCamera = () => {
  if (mediaStream.value) {
    mediaStream.value.getTracks().forEach((track) => track.stop())
    mediaStream.value = null
  }

  if (videoRef.value) {
    videoRef.value.srcObject = null
  }

  error.value = ''
}

// 切换摄像头
const handleSwitchCamera = () => {
  currentCamera.value =
    currentCamera.value === 'environment' ? 'user' : 'environment'
  startCamera()
}

// 拍照
const handleCapture = () => {
  if (!videoRef.value || !canvasRef.value || error.value) {
    return
  }

  const video = videoRef.value
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')

  if (!ctx) {
    return
  }

  // 设置 canvas 尺寸与视频一致
  canvas.width = video.videoWidth
  canvas.height = video.videoHeight

  // 绘制视频帧到 canvas
  ctx.drawImage(video, 0, 0, canvas.width, canvas.height)

  // 转换为图片
  const imageData = canvas.toDataURL('image/jpeg', 0.9)
  emit('capture', imageData)
}

// 关闭相机
const handleClose = () => {
  emit('close')
}

// 组件卸载时清理
onBeforeUnmount(() => {
  stopCamera()
})
</script>

<style scoped>
.camera-modal {
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

.camera-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.camera-header {
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

.camera-title {
  color: white;
  font-size: 16px;
  font-weight: 500;
}

.camera-preview {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
}

.camera-preview video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.camera-error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
  padding: 20px;
}

.error-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.camera-error p {
  font-size: 14px;
  margin: 0 0 16px 0;
  opacity: 0.9;
}

.retry-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  padding: 10px 24px;
  border-radius: 20px;
  font-size: 14px;
  cursor: pointer;
}

.camera-controls {
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

.capture-btn {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  border: 4px solid white;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s;
}

.capture-btn:active {
  transform: scale(0.9);
}

.capture-btn:disabled {
  opacity: 0.5;
}

.capture-inner {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: white;
}
</style>
