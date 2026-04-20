<template>
  <div class="review-page">
    <div class="progress-section">
      <div class="progress-header">
        <h2>今日复习</h2>
        <span class="progress-text">{{ currentIndex + 1 }} / {{ reviewList.length }}</span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
      </div>
    </div>

    <div v-if="currentError" class="review-card">
      <div class="card-header">
        <span class="subject-tag">{{ currentError.subjectName }}</span>
        <span class="difficulty-tag" :class="currentError.difficulty">{{ currentError.difficultyName }}</span>
      </div>

      <div class="question-section">
        <h3>题目</h3>
        <p class="question-content">{{ currentError.content }}</p>
      </div>

      <div class="answer-section">
        <h3>你的答案</h3>
        <textarea v-model="userAnswer" placeholder="输入你的答案..." rows="4"></textarea>
      </div>

      <div class="reference-section" v-if="showReference">
        <h3>参考答案</h3>
        <p class="reference-content">{{ currentError.answer }}</p>
      </div>

      <div class="action-buttons">
        <button v-if="!showReference" class="btn primary" @click="showReference = true">查看答案</button>
        <template v-else>
          <button class="btn wrong" @click="markWrong">没掌握</button>
          <button class="btn correct" @click="markCorrect">已掌握</button>
        </template>
      </div>
    </div>

    <div v-else class="complete-state">
      <div class="complete-icon">🎉</div>
      <h2>今日复习完成！</h2>
      <p class="complete-stats">
        正确率：{{ correctCount }} / {{ totalCount }}
      </p>
      <button class="btn primary" @click="resetReview">重新开始</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const currentIndex = ref(0)
const userAnswer = ref('')
const showReference = ref(false)
const correctCount = ref(0)
const totalCount = ref(0)

const reviewList = ref([
  {
    id: 1,
    subject: 'math',
    subjectName: '数学',
    difficulty: 'medium',
    difficultyName: '中等',
    content: '求函数 f(x) = x² - 2x + 1 的最小值',
    answer: '最小值为 0，当 x = 1 时取得'
  },
  {
    id: 2,
    subject: 'physics',
    subjectName: '物理',
    difficulty: 'hard',
    difficultyName: '困难',
    content: '一个质量为 2kg 的物体从 10m 高处自由落下，求落地时的速度（g = 10m/s²）',
    answer: 'v = √(2gh) = √(2×10×10) = √200 = 10√2 ≈ 14.14 m/s'
  },
  {
    id: 3,
    subject: 'english',
    subjectName: '英语',
    difficulty: 'easy',
    difficultyName: '简单',
    content: '翻译句子：The quick brown fox jumps over the lazy dog',
    answer: '敏捷的棕色狐狸跳过了懒惰的狗'
  }
])

const currentError = computed(() => {
  return reviewList.value[currentIndex.value]
})

const progressPercent = computed(() => {
  if (reviewList.value.length === 0) return 0
  return ((currentIndex.value + 1) / reviewList.value.length) * 100
})

const markWrong = () => {
  totalCount.value++
  nextQuestion()
}

const markCorrect = () => {
  correctCount.value++
  totalCount.value++
  nextQuestion()
}

const nextQuestion = () => {
  showReference.value = false
  userAnswer.value = ''
  if (currentIndex.value < reviewList.value.length - 1) {
    currentIndex.value++
  }
}

const resetReview = () => {
  currentIndex.value = 0
  userAnswer.value = ''
  showReference.value = false
  correctCount.value = 0
  totalCount.value = 0
}
</script>

<style scoped>
.review-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  max-width: 70%;
  margin: 0 auto;
}

.progress-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-header h2 {
  font-size: 18px;
  margin: 0;
  color: var(--text-primary);
}

.progress-text {
  font-size: 14px;
  color: var(--text-secondary);
}

.progress-bar {
  height: 8px;
  background: var(--border-color);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), #42a5f5);
  transition: width 0.3s;
}

.review-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.card-header {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.subject-tag {
  padding: 4px 8px;
  background: var(--primary-light);
  color: var(--primary-color);
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.difficulty-tag {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.difficulty-tag.easy {
  background: #e8f5e9;
  color: #43a047;
}

.difficulty-tag.medium {
  background: #fff3e0;
  color: #e65100;
}

.difficulty-tag.hard {
  background: #ffebee;
  color: #c62828;
}

.question-section,
.answer-section,
.reference-section {
  margin-bottom: 20px;
}

.question-section h3,
.answer-section h3,
.reference-section h3 {
  font-size: 16px;
  margin: 0 0 12px 0;
  color: var(--text-primary);
}

.question-content,
.reference-content {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary);
  margin: 0;
  padding: 12px;
  background: var(--input-bg);
  border-radius: 8px;
}

.answer-section textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  background: var(--input-bg);
  color: var(--text-primary);
  resize: vertical;
  box-sizing: border-box;
}

.answer-section textarea:focus {
  outline: none;
  border-color: var(--primary-color);
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
}

.btn.primary {
  background: var(--primary-color);
  color: white;
}

.btn.correct {
  background: #43a047;
  color: white;
}

.btn.wrong {
  background: #e65100;
  color: white;
}

.btn:active {
  transform: scale(0.98);
}

.complete-state {
  text-align: center;
  padding: 60px 20px;
}

.complete-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.complete-state h2 {
  font-size: 24px;
  margin: 0 0 12px 0;
  color: var(--text-primary);
}

.complete-stats {
  font-size: 18px;
  color: var(--text-secondary);
  margin-bottom: 24px;
}
</style>