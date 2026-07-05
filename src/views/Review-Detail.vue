<template>
  <div class="review-detail-page">
    <!-- 顶部 -->
    <div class="top-bar">
      <button class="back-btn" @click="exitReview">← 退出</button>
      <span class="top-title">复习</span>
      <span class="top-count">{{ currentIndex + 1 }} / {{ queue.length }}</span>
    </div>

    <!-- 进度条 -->
    <div class="progress-bar">
      <div
        class="progress-fill"
        :style="{ width: progressPercent + '%' }"
      ></div>
    </div>

    <div v-if="currentCard" class="review-content">
      <!-- 科目标签 -->
      <div class="card-meta">
        <span
          class="subject-tag"
          :style="getSubjectStyle(currentCard.question)"
          >{{ subjectName }}</span
        >
        <span class="difficulty-info"
          >记忆强度: {{ currentCard.srs.stability?.toFixed(1) }}天</span
        >
      </div>

      <!-- 题目区 (始终可见, MarkdownTextarea 预览模式) -->
      <div class="section-block">
        <div class="section-label">题目</div>
        <MarkdownTextarea
          v-model="promptText"
          :showPreview="true"
          :defaultViewMode="'preview'"
          :previewTitle="''"
        />
      </div>

      <!-- 显示答案按钮 / 答案区 -->
      <div v-if="!showAnswer" class="action-row">
        <button class="btn-reveal" @click="showAnswer = true">
          <Icon name="message-square" :size="18" /> 显示答案
        </button>
      </div>

      <template v-if="showAnswer">
        <!-- 答案区 -->
        <div class="section-block" v-if="answerText">
          <div class="section-label">参考答案</div>
          <MarkdownTextarea
            v-model="answerText"
            :showPreview="true"
            :defaultViewMode="'preview'"
            :previewTitle="''"
          />
        </div>

        <!-- 解析区 -->
        <div class="section-block" v-if="analysisText">
          <div class="section-label">解析</div>
          <MarkdownTextarea
            v-model="analysisText"
            :showPreview="true"
            :defaultViewMode="'preview'"
            :previewTitle="''"
          />
        </div>

        <!-- 链接 -->
        <div class="action-row">
          <button class="btn-link" @click="goToDetail">
            <Icon name="link" :size="16" /> 查看/编辑详细
          </button>
        </div>

        <!-- 滑动条评分 -->
        <div class="slider-section">
          <div class="slider-labels">
            <span>完全忘了</span>
            <span>模糊</span>
            <span>基本对了</span>
            <span>完美记住</span>
          </div>

          <div class="slider-container">
            <div
              class="slider-wrapper"
              @mouseenter="sScale = 1.08"
              @mouseleave="
                sScale = 1,
                sOverflow = 0,
                sRegion = 'middle'
              "
              @touchstart.passive="sScale = 1.08"
              @touchend="
                sScale = 1,
                sOverflow = 0,
                sRegion = 'middle',
                submitReview()
              "
              :style="{
                transform: `scale(${sScale})`,
                opacity: 0.7 + (0.3 * (sScale - 1)) / 0.2
              }"
            >
              <!-- 滑块轨道 -->
              <div
                ref="sliderRef"
                class="slider-root"
                @pointermove="onSliderMove"
                @pointerdown="onSliderDown"
                @pointerup="onSliderUp"
                @pointercancel="onSliderUp"
                @lostpointercapture="onSliderUp"
              >
                <div
                  class="slider-track-fx"
                  :style="{
                    transform: `scaleX(${sTrackScaleX}) scaleY(${sTrackScaleY})`,
                    transformOrigin: sTrackOrigin,
                    height: sTrackHeight,
                    transition: 'transform 0.05s linear, height 0.2s'
                  }"
                >
                  <div class="slider-track-bg">
                    <div
                      class="slider-range"
                      :style="{
                        width: sRangePercent + '%',
                        background: sRangeColor
                      }"
                    />
                  </div>
                </div>
              </div>
            </div>

            <div class="slider-value-row">
              <span class="feedback-tag" :style="{ color: feedbackColor }">{{
                feedbackLabel
              }}</span>
              <span class="value-indicator">{{
                Math.round(feedbackValue * 100)
              }}</span>
            </div>
          </div>
        </div>

        <!-- SRS Debug -->
        <div class="srs-debug">
          <div class="debug-toggle" @click="showDebug = !showDebug">
            SRS Details {{ showDebug ? '▲' : '▼' }}
          </div>
          <div v-if="showDebug" class="debug-body">
            <div class="debug-row">
              <span>stability</span><span>{{ currentCard.srs.stability }}</span>
            </div>
            <div class="debug-row">
              <span>difficulty</span
              ><span>{{ currentCard.srs.difficulty }}</span>
            </div>
            <div class="debug-row">
              <span>recall_rate</span
              ><span>{{ currentCard.srs.recall_rate }}</span>
            </div>
            <div class="debug-row">
              <span>review_count</span
              ><span>{{ currentCard.srs.review_count }}</span>
            </div>
            <div class="debug-row">
              <span>next_review_at</span
              ><span>{{ formatTs(currentCard.srs.next_review_at) }}</span>
            </div>
            <div class="debug-row">
              <span>last_review_at</span
              ><span>{{ formatTs(currentCard.srs.last_review_at) }}</span>
            </div>
            <div class="debug-row" v-if="lastResult">
              <span>→ new_stability</span
              ><span>{{ lastResult.new_stability?.toFixed(2) }}</span>
            </div>
            <div class="debug-row" v-if="lastResult">
              <span>→ new_difficulty</span
              ><span>{{ lastResult.new_difficulty?.toFixed(2) }}</span>
            </div>
            <div class="debug-row" v-if="lastResult">
              <span>→ next_interval</span
              ><span>{{ lastResult.next_interval_days?.toFixed(1) }} 天</span>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-illustration">
      <div class="empty-icon"></div>
      <div class="empty-title">没有待复习的题目</div>
      <div class="empty-desc">所有错题都已复习完毕，做得很棒！</div>
      <button class="btn-back" @click="exitReview" style="margin-top: 16px">
        返回
      </button>
    </div>

    <!-- 提交中遮罩 -->
    <div v-if="submitting" class="submitting-overlay">
      <div class="submitting-spinner"></div>
      <p>提交中...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { submitReviewResult } from '../apis/srs'
import { getReviewQueue, clearReviewQueue } from '../services/reviewStore'
import type { ReviewOutput } from '../apis/srs'

const router = useRouter()

// ============ State ============
const queue = getReviewQueue()
const currentIndex = ref(0)
const showAnswer = ref(false)
const feedbackValue = ref(0.5)
const showDebug = ref(false)
const submitting = ref(false)
const lastResult = ref<ReviewOutput | null>(null)

// ============ 滑动条交互状态 ============
const MAX_OVERFLOW = 50
const sliderRef = ref<HTMLElement | null>(null)
const sRegion = ref<'left' | 'middle' | 'right'>('middle')
const sOverflow = ref(0)
const sScale = ref(1)
// 缓存轨道尺寸，避免每次 move 都 getBoundingClientRect
let _sliderLeft = 0
let _sliderWidth = 1

function sDecay(value: number, max: number) {
  if (max === 0) return 0
  const entry = value / max
  const sigmoid = 2 * (1 / (1 + Math.exp(-entry)) - 0.5)
  return sigmoid * max
}

const sRangePercent = computed(() => feedbackValue.value * 100)

function lerpColor(t: number): string {
  if (t <= 0) return '#f44336'
  if (t >= 1) return '#4caf50'
  if (t < 0.33) {
    const p = t / 0.33
    const r = 244 + (255 - 244) * p
    const g = 51 + (152 - 51) * p
    const b = 54 + (7 - 54) * p
    return `rgb(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)})`
  } else if (t < 0.66) {
    const p = (t - 0.33) / 0.33
    const r = 255
    const g = 152 + (193 - 152) * p
    const b = 7
    return `rgb(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)})`
  } else {
    const p = (t - 0.66) / 0.34
    const r = 255 + (76 - 255) * p
    const g = 193 + (175 - 193) * p
    const b = 7 + (80 - 7) * p
    return `rgb(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)})`
  }
}

const sRangeColor = computed(() => lerpColor(feedbackValue.value))

const sTrackScaleX = computed(() => {
  return 1 + sOverflow.value / _sliderWidth
})

const sTrackScaleY = computed(() => {
  return 1 - (sOverflow.value / MAX_OVERFLOW) * 0.2
})

const sTrackOrigin = computed(() => {
  // 基于缓存的 left，不需要 getBoundingClientRect
  return 'center'
})

const sTrackHeight = computed(() => {
  const base = 14
  const max = 20
  const h = base + ((max - base) * (sScale.value - 1)) / 0.2
  return h + 'px'
})

function onSliderDown(e: PointerEvent) {
  // 缓存轨道尺寸，避免 move 时反复 reflow
  const rect = sliderRef.value?.getBoundingClientRect()
  if (rect) {
    _sliderLeft = rect.left
    _sliderWidth = rect.width
  }
  onSliderMove(e)
  ;(e.currentTarget as HTMLElement)?.setPointerCapture(e.pointerId)
}

function onSliderMove(e: PointerEvent) {
  if (e.buttons === 0) return
  // 使用缓存的 left/width，不调用 getBoundingClientRect
  let newVal = (e.clientX - _sliderLeft) / _sliderWidth
  newVal = Math.min(Math.max(newVal, 0), 1)
  feedbackValue.value = newVal
  // 溢出计算
  const rightEdge = _sliderLeft + _sliderWidth
  if (e.clientX < _sliderLeft) {
    sRegion.value = 'left'
    sOverflow.value = sDecay(_sliderLeft - e.clientX, MAX_OVERFLOW)
  } else if (e.clientX > rightEdge) {
    sRegion.value = 'right'
    sOverflow.value = sDecay(e.clientX - rightEdge, MAX_OVERFLOW)
  } else {
    sRegion.value = 'middle'
    sOverflow.value = 0
  }
}

function onSliderUp(_e: PointerEvent) {
  sOverflow.value = 0
  sRegion.value = 'middle'
  submitReview()
}

// ============ Computed ============
const currentCard = computed(() => queue[currentIndex.value] || null)

// MarkdownTextarea 需要 writable refs
const promptText = ref('')
const answerText = ref('')
const analysisText = ref('')

const subjectName = computed(() => currentCard.value?.subjectName || '')

watch(
  currentCard,
  (card) => {
    if (card) {
      promptText.value = card.question?.prompt || ''
      answerText.value = card.question?.answer || ''
      analysisText.value = card.question?.analysis || ''
    }
  },
  { immediate: true }
)

const progressPercent = computed(() => {
  if (queue.length === 0) return 0
  return ((currentIndex.value + 1) / queue.length) * 100
})

const feedbackColor = computed(() => {
  const v = feedbackValue.value
  if (v < 0.2) return '#f44336'
  if (v < 0.33) return '#ff9800'
  if (v < 0.67) return '#ffc107'
  return '#4caf50'
})

const feedbackLabel = computed(() => {
  const v = feedbackValue.value
  if (v < 0.2) return '完全忘了'
  if (v < 0.33) return '模糊'
  if (v < 0.67) return '基本对了'
  if (v < 1.0) return '基本对了'
  return '完美记住'
})

// ============ Methods ============
function getSubjectStyle(_question: any) {
  return { backgroundColor: '#e3f2fd', color: '#1976d2' }
}

function formatTs(ts: number | null | undefined): string {
  if (!ts) return '-'
  return new Date(ts * 1000).toLocaleString('zh-CN')
}

async function submitReview() {
  if (submitting.value || !currentCard.value) return
  submitting.value = true
  try {
    const result = await submitReviewResult({
      question_id: currentCard.value.questionId,
      feedback: feedbackValue.value
    })
    lastResult.value = result

    // 延迟后进入下一题
    await new Promise((r) => setTimeout(r, 400))

    if (currentIndex.value < queue.length - 1) {
      currentIndex.value++
      showAnswer.value = false
      feedbackValue.value = 0.5
      showDebug.value = false
      lastResult.value = null
    } else {
      clearReviewQueue()
      router.replace({ name: 'Preview' })
    }
  } catch (e) {
    console.error('提交复习结果失败:', e)
    alert('提交失败: ' + e)
  } finally {
    submitting.value = false
  }
}

function goToDetail() {
  if (!currentCard.value) return
  router.push({
    name: 'ManageDetail',
    params: { id: currentCard.value.questionId }
  })
}

function exitReview() {
  clearReviewQueue()
  router.replace({ name: 'Preview' })
}

// ============ Lifecycle ============
onMounted(() => {
  if (queue.length === 0) {
    router.replace({ name: 'Preview' })
  }
})
</script>

<style scoped>
.review-detail-page {
  padding: 16px;
  padding-bottom: 40px;
  background: var(--bg-primary);
  min-height: 100vh;
}

/* ===== Top Bar ===== */
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.back-btn {
  background: none;
  border: none;
  color: var(--primary-color);
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
}
.top-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}
.top-count {
  font-size: 14px;
  color: var(--text-secondary);
}

/* ===== Progress Bar ===== */
.progress-bar {
  height: 6px;
  background: var(--border-color);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 20px;
}
.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), #42a5f5);
  transition: width 0.4s ease;
  border-radius: 3px;
}

/* ===== Card Meta ===== */
.card-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}
.subject-tag {
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}
.difficulty-info {
  font-size: 12px;
  color: var(--text-secondary);
}

/* ===== Section Block ===== */
.section-block {
  margin-bottom: 16px;
}
.section-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* ===== Action Row ===== */
.action-row {
  text-align: center;
  margin-bottom: 16px;
}

.btn-reveal {
  width: 100%;
  padding: 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 12px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 4px 12px rgba(25, 118, 210, 0.3);
}
.btn-reveal:active {
  transform: scale(0.98);
  box-shadow: 0 2px 6px rgba(25, 118, 210, 0.2);
}

.btn-link {
  padding: 10px 20px;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--primary-color);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-link:hover {
  background: var(--primary-light);
  border-color: var(--primary-color);
}

/* ===== Slider Section ===== */
.slider-section {
  margin: 24px 0 16px;
  padding: 20px 16px;
  background: var(--card-bg);
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  margin-bottom: 16px;
  font-size: 11px;
  color: var(--text-secondary);
  padding: 0 4px;
}

/* ===== 新滑动条 ===== */
.slider-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.slider-wrapper {
  display: flex;
  width: 100%;
  touch-action: none;
  user-select: none;
  align-items: center;
  justify-content: center;
  transition:
    transform 0.2s ease,
    opacity 0.2s ease;
}
.slider-root {
  position: relative;
  display: flex;
  flex: 1;
  max-width: none;
  cursor: grab;
  touch-action: none;
  user-select: none;
  align-items: center;
  padding: 1.2rem 0;
}

.slider-root:active {
  cursor: grabbing;
}

.slider-track-fx {
  display: flex;
  flex: 1;
  height: 14px;
  will-change: transform;
}

.slider-track-bg {
  position: relative;
  height: 100%;
  flex: 1;
  overflow: hidden;
  border-radius: 9999px;
  background: var(--bg-tertiary);
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
}

.slider-range {
  position: absolute;
  height: 100%;
  border-radius: 9999px;
  transition: width 0.05s linear;
}

.slider-value-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.feedback-tag {
  font-size: 13px;
  font-weight: 600;
}

.value-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 36px;
  padding: 2px 10px;
  border-radius: 999px;
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

/* ===== SRS Debug ===== */
.srs-debug {
  margin-bottom: 16px;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}
.debug-toggle {
  padding: 10px 14px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
  background: var(--bg-secondary);
}
.debug-toggle:hover {
  background: var(--border-color);
}
.debug-body {
  padding: 10px 14px;
  background: var(--bg-secondary);
  font-family: var(--font-family-mono);
  font-size: 12px;
}
.debug-row {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
  color: var(--text-primary);
}
.debug-row span:first-child {
  color: var(--text-secondary);
}

.debug-row span:last-child {
  color: var(--primary-color);
  font-weight: 500;
}

/* ===== Empty State ===== */
.empty-state {
  text-align: center;
  padding: 80px 20px;
  color: var(--text-secondary);
}
.empty-state p {
  font-size: 16px;
  margin-bottom: 20px;
}
.btn-back {
  padding: 12px 32px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
}

/* ===== Submitting Overlay ===== */
.submitting-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 999;
  color: white;
  font-size: 16px;
}
.submitting-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 16px;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
