<template>
  <div class="review-detail-page">
    <!-- 顶部 -->
    <div class="top-bar">
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="back-btn"
        @click="exitReview"
      >
        ← 退出
      </q-btn>
      <span class="top-title">复习</span>
      <span class="top-count">{{ currentIndex + 1 }} / {{ queue.length }}</span>
    </div>

    <q-linear-progress
      :value="progressPercent / 100"
      rounded
      color="primary"
      size="6px"
      aria-label="复习进度"
    />
    <div v-if="currentCard" class="review-content">
      <!-- 科目标签 -->
      <div class="card-meta">
        <span class="subject-tag" :style="getSubjectStyle()">{{
          subjectName
        }}</span>
        <span class="difficulty-info"
          >记忆强度: {{ currentCard.srs.stability?.toFixed(1) }}天</span
        >
      </div>

      <!-- 题目区 (始终可见, MarkdownTextarea 预览模式) -->
      <q-card flat bordered class="section-block">
        <div class="section-label">题目</div>
        <MarkdownTextarea
          v-model="promptText"
          :show-preview="true"
          readonly
          :default-view-mode="'preview'"
          :preview-title="''"
        />
      </q-card>

      <!-- 显示答案按钮 / 答案区 -->
      <div v-if="!showAnswer" class="action-row">
        <q-btn
          no-caps
          unelevated
          type="button"
          color="primary"
          class="btn-reveal"
          @click="showAnswer = true"
        >
          <Icon name="message-square" :size="18" /> 显示答案
        </q-btn>
      </div>

      <template v-if="showAnswer">
        <!-- 答案区 -->
        <q-card v-if="answerText" flat bordered class="section-block">
          <div class="section-label">参考答案</div>
          <MarkdownTextarea
            v-model="answerText"
            :show-preview="true"
            readonly
            :default-view-mode="'preview'"
            :preview-title="''"
          />
        </q-card>

        <!-- 解析区 -->
        <q-card v-if="analysisText" flat bordered class="section-block">
          <div class="section-label">解析</div>
          <MarkdownTextarea
            v-model="analysisText"
            :show-preview="true"
            readonly
            :default-view-mode="'preview'"
            :preview-title="''"
          />
        </q-card>

        <!-- 链接 -->
        <div class="action-row">
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="btn-link"
            @click="goToDetail"
          >
            <Icon name="link" :size="16" /> 查看/编辑详细
          </q-btn>
        </div>

        <q-card flat bordered class="slider-section">
          <h3>这道题，你掌握得怎样？</h3>
          <div class="slider-labels">
            <span>完全忘了</span><span>模糊</span><span>基本对了</span
            ><span>完美记住</span>
          </div>
          <q-slider
            v-model="feedbackValue"
            :min="0"
            :max="1"
            :step="0.01"
            :disable="submitting"
            color="primary"
            label
            :label-value="Math.round(feedbackValue * 100)"
            aria-label="掌握程度"
          />
          <div class="slider-value-row">
            <span class="feedback-tag" :style="{ color: feedbackColor }">{{
              feedbackLabel
            }}</span
            ><span>{{ Math.round(feedbackValue * 100) }} / 100</span>
          </div>
          <q-btn
            class="full-width"
            unelevated
            color="primary"
            label="提交并继续"
            :loading="submitting"
            @click="submitReview"
          />
        </q-card>

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
              <span>retrievability</span
              ><span>{{ currentCard.srs.retrievability }}</span>
            </div>
            <div class="debug-row">
              <span>reviewCount</span
              ><span>{{ currentCard.srs.reviewCount }}</span>
            </div>
            <div class="debug-row">
              <span>nextReviewAt</span
              ><span>{{ formatTs(currentCard.srs.nextReviewAt) }}</span>
            </div>
            <div class="debug-row">
              <span>lastReviewAt</span
              ><span>{{ formatTs(currentCard.srs.lastReviewAt) }}</span>
            </div>
            <div v-if="lastResult" class="debug-row">
              <span>→ new_stability</span
              ><span>{{ lastResult.srs.stability?.toFixed(2) }}</span>
            </div>
            <div v-if="lastResult" class="debug-row">
              <span>→ new_difficulty</span
              ><span>{{ lastResult.srs.difficulty?.toFixed(2) }}</span>
            </div>
            <div v-if="lastResult" class="debug-row">
              <span>→ next_interval</span
              ><span>{{ lastResult.nextIntervalDays?.toFixed(1) }} 天</span>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-illustration">
      <div class="empty-icon" />
      <div class="empty-title">没有待复习的题目</div>
      <div class="empty-desc">所有错题都已复习完毕，做得很棒！</div>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="btn-back"
        style="margin-top: 16px"
        @click="exitReview"
      >
        返回
      </q-btn>
    </div>

    <q-inner-loading
      :showing="submitting"
      label="正在保存复习结果…"
      color="primary"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import { submitReview as submitReviewRequest } from '../api'
import { useLatestRequest } from '../composables/useLatestRequest'
import { clearReviewQueue, getReviewQueue } from '../services/reviewStore'
import type { SubmitReviewResponse } from '../types'
import { showAlert } from '../utils/dialog'
import { formatTimestamp as formatTs } from '../utils/questionDisplay'

const router = useRouter()

// ============ State ============
const queue = getReviewQueue()
const currentIndex = ref(0)
const showAnswer = ref(false)
const feedbackValue = ref(0.5)
const showDebug = ref(false)
const submitting = ref(false)
const lastResult = ref<SubmitReviewResponse | null>(null)
const beginSubmit = useLatestRequest()

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
      promptText.value = card.question?.stem || ''
      answerText.value = card.question?.correctAnswer || ''
      analysisText.value = card.question?.explanation || ''
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
function getSubjectStyle() {
  return { backgroundColor: '#e3f2fd', color: '#1976d2' }
}

async function submitReview() {
  if (submitting.value || !currentCard.value) return
  const isCurrent = beginSubmit()
  submitting.value = true
  try {
    const result = await submitReviewRequest({
      questionId: currentCard.value.questionId,
      feedback: feedbackValue.value,
      reviewedAt: new Date().toISOString()
    })
    if (!isCurrent()) return
    lastResult.value = result
    currentCard.value.srs = result.srs

    // 延迟后进入下一题
    await new Promise((r) => setTimeout(r, 400))
    if (!isCurrent()) return

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
    if (!isCurrent()) return
    console.error('提交复习结果失败:', e)
    showAlert('提交失败: ' + e)
  } finally {
    if (isCurrent()) submitting.value = false
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
