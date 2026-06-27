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
          >稳定性: {{ currentCard.srs.stability?.toFixed(1) }}天</span
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
          <button class="btn-link" @click="goToDetail"><Icon name="link" :size="16" /> 查看/编辑详细</button>
        </div>

        <!-- 滑动条评分 -->
        <div class="slider-section">
          <div class="slider-labels">
            <span>完全忘了</span>
            <span>模糊</span>
            <span>基本对了</span>
            <span>完美记住</span>
          </div>
          <div class="slider-track-wrapper">
            <input
              type="range"
              min="0"
              max="1"
              step="0.000000000001"
              v-model.number="feedbackValue"
              @touchend="submitReview"
              @mouseup="submitReview"
              class="gradient-slider"
            />
          </div>
          <div class="slider-value">
            反馈值: <strong>{{ feedbackValue.toFixed(2) }}</strong>
            <span class="feedback-tag" :style="{ color: feedbackColor }">{{
              feedbackLabel
            }}</span>
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
    <div v-else class="empty-state">
      <p>没有待复习的题目</p>
      <button class="btn-back" @click="exitReview">返回</button>
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
  if (v < 0.4) return '#ff9800'
  if (v < 0.7) return '#ffc107'
  return '#4caf50'
})

const feedbackLabel = computed(() => {
  const v = feedbackValue.value
  if (v < 0.15) return '完全忘了'
  if (v < 0.35) return '模糊'
  if (v < 0.65) return '基本对了'
  if (v < 0.9) return '大部分对了'
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
  padding: 16px;
  background: var(--card-bg);
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 11px;
  color: var(--text-secondary);
}

.slider-track-wrapper {
  padding: 4px 0;
}

.gradient-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 10px;
  border-radius: 5px;
  outline: none;
  background: linear-gradient(to right, #f44336, #ff9800, #ffc107, #4caf50);
  cursor: pointer;
}

.gradient-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: white;
  border: 3px solid var(--primary-color);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  cursor: pointer;
  transition: transform 0.15s;
}
.gradient-slider::-webkit-slider-thumb:active {
  transform: scale(1.15);
}

.gradient-slider::-moz-range-thumb {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: white;
  border: 3px solid var(--primary-color);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  cursor: pointer;
}

.slider-value {
  text-align: center;
  margin-top: 12px;
  font-size: 14px;
  color: var(--text-primary);
}
.feedback-tag {
  margin-left: 8px;
  font-weight: 600;
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
  background: #1a1a2e;
  font-family: 'Courier New', monospace;
  font-size: 12px;
}
.debug-row {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
  color: #e0e0e0;
}
.debug-row span:first-child {
  color: #7ec8e3;
}
.debug-row span:last-child {
  color: #a8e6cf;
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
