<template>
  <div class="camera-modal" v-if="visible">
    <div class="camera-container">
      <!-- 顶部操作栏 -->
      <div class="camera-header">
        <button class="header-btn" @click="handleClose">
          <Icon name="x" :size="18" />
        </button>
        <span class="camera-title">拍照</span>
        <button
          class="header-btn"
          @click="handleSwitchCamera"
          v-if="hasMultipleCameras"
        >
          <Icon name="refresh-cw" :size="18" />
        </button>
        <div class="header-placeholder" v-else></div>
      </div>

      <!-- 预览区 -->
      <div class="camera-preview">
        <video ref="videoRef" autoplay playsinline></video>
        <canvas ref="canvasRef" style="display: none"></canvas>

        <!-- 取景框装饰 -->
        <div class="viewfinder">
          <div class="vf-corner tl"></div>
          <div class="vf-corner tr"></div>
          <div class="vf-corner bl"></div>
          <div class="vf-corner br"></div>
        </div>

        <!-- 错误提示 -->
        <div class="camera-error" v-if="error">
          <Icon name="triangle-alert" :size="32" class="error-icon" />
          <p>{{ error }}</p>
          <button class="retry-btn" @click="startCamera">重试</button>
        </div>
      </div>

      <!-- 底部拍摄按钮 -->
      <div class="camera-controls">
        <button class="capture-btn" @click="handleCapture" :disabled="!!error">
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

watch(
  () => props.visible,
  async (newVal) => {
    if (newVal) await startCamera()
    else stopCamera()
  }
)

const startCamera = async () => {
  error.value = ''
  try {
    if (mediaStream.value) {
      mediaStream.value.getTracks().forEach((track) => track.stop())
    }
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
    if (videoRef.value) videoRef.value.srcObject = stream
    const devices = await navigator.mediaDevices.enumerateDevices()
    hasMultipleCameras.value =
      devices.filter((d) => d.kind === 'videoinput').length > 1
  } catch (err: any) {
    console.error('相机启动失败:', err)
    if (err.name === 'NotAllowedError') error.value = '请允许访问摄像头权限'
    else if (err.name === 'NotFoundError') error.value = '未找到摄像头设备'
    else if (err.name === 'NotReadableError')
      error.value = '摄像头被其他应用占用'
    else error.value = '相机启动失败，请检查设备'
    emit('error')
  }
}

const stopCamera = () => {
  if (mediaStream.value) {
    mediaStream.value.getTracks().forEach((track) => track.stop())
    mediaStream.value = null
  }
  if (videoRef.value) videoRef.value.srcObject = null
  error.value = ''
}

const handleSwitchCamera = () => {
  currentCamera.value =
    currentCamera.value === 'environment' ? 'user' : 'environment'
  startCamera()
}

const handleCapture = () => {
  if (!videoRef.value || !canvasRef.value || error.value) return
  const video = videoRef.value
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  canvas.width = video.videoWidth
  canvas.height = video.videoHeight
  ctx.drawImage(video, 0, 0, canvas.width, canvas.height)
  emit('capture', canvas.toDataURL('image/jpeg', 0.9))
}

const handleClose = () => emit('close')

onBeforeUnmount(() => stopCamera())
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

.camera-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}

/* ===== 顶部栏 ===== */
.camera-header {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 16px 20px;
  padding-top: calc(16px + env(safe-area-inset-top));
  background: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0.6) 0%,
    transparent 100%
  );
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

.header-placeholder {
  width: 40px;
}

.camera-title {
  color: #fff;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

/* ===== 预览区 ===== */
.camera-preview {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  overflow: hidden;
}

.camera-preview video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ===== 取景框 ===== */
.viewfinder {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 80%;
  max-width: 360px;
  aspect-ratio: 3/4;
  pointer-events: none;
}

.vf-corner {
  position: absolute;
  width: 24px;
  height: 24px;
  border-color: rgba(255, 255, 255, 0.5);
  border-style: solid;
}
.vf-corner.tl {
  top: 0;
  left: 0;
  border-width: 2px 0 0 2px;
}
.vf-corner.tr {
  top: 0;
  right: 0;
  border-width: 2px 2px 0 0;
}
.vf-corner.bl {
  bottom: 0;
  left: 0;
  border-width: 0 0 2px 2px;
}
.vf-corner.br {
  bottom: 0;
  right: 0;
  border-width: 0 2px 2px 0;
}

/* ===== 错误提示 ===== */
.camera-error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: #fff;
  padding: 24px 32px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 16px;
  backdrop-filter: blur(12px);
}

.error-icon {
  margin-bottom: 12px;
  opacity: 0.8;
}

.camera-error p {
  font-size: 14px;
  margin: 0 0 16px;
  opacity: 0.9;
  line-height: 1.5;
}

.retry-btn {
  padding: 10px 28px;
  background: var(--primary-color, #1976d2);
  color: #fff;
  border: none;
  border-radius: 20px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.retry-btn:hover {
  opacity: 0.85;
  transform: scale(1.02);
}

/* ===== 底部拍摄按钮 ===== */
.camera-controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 24px;
  padding-bottom: calc(24px + env(safe-area-inset-bottom));
  background: linear-gradient(to top, rgba(0, 0, 0, 0.6) 0%, transparent 100%);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10;
}

.capture-btn {
  width: 76px;
  height: 76px;
  border-radius: 50%;
  border: 4px solid rgba(255, 255, 255, 0.9);
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s ease;
}

.capture-btn:hover {
  transform: scale(1.05);
}

.capture-btn:active {
  transform: scale(0.88);
}

.capture-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}

.capture-inner {
  width: 58px;
  height: 58px;
  border-radius: 50%;
  background: #fff;
  transition: all 0.2s;
}

.capture-btn:active .capture-inner {
  background: rgba(255, 255, 255, 0.8);
}
</style>
