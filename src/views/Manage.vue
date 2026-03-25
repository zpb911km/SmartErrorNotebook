<template>
  <div class="manage-page">
    <div class="filter-bar">
      <select v-model="filters.subject" class="filter-select">
        <option value="">全部科目</option>
        <option value="math">数学</option>
        <option value="physics">物理</option>
        <option value="chemistry">化学</option>
        <option value="english">英语</option>
      </select>

      <select v-model="filters.status" class="filter-select">
        <option value="">全部状态</option>
        <option value="pending">待复习</option>
        <option value="reviewed">已复习</option>
        <option value="mastered">已掌握</option>
      </select>

      <div class="search-box" :class="{ 'blinking': isSearchBlinking }">
        <input 
          type="text" 
          v-model="filters.keyword" 
          placeholder="搜索错题..."
          ref="searchInputRef"
          @focus="onSearchFocus"
          @click="onSearchFocus"
        >
      </div>
    </div>

    <div class="error-list">
      <div v-for="error in filteredErrors" :key="error.id" class="error-card" @click="viewError(error)">
        <div class="error-header">
          <span class="subject-tag" :class="error.subject">{{ error.subjectName }}</span>
          <span class="difficulty-tag" :class="error.difficulty">{{ error.difficultyName }}</span>
        </div>
        <div class="error-content">{{ error.content }}</div>
        <div class="error-footer">
          <span class="error-date">{{ error.date }}</span>
          <span class="error-status" :class="error.status">{{ error.statusText }}</span>
        </div>
      </div>
    </div>

    <div v-if="filteredErrors.length === 0" class="empty-state">
      <div class="empty-icon">📭</div>
      <p>暂无错题</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { getQuestions } from '../apis/errorQuestions'
import { getSubjects } from '../apis/subjects'

const route = useRoute()

const filters = ref({
  subject: '',
  status: '',
  keyword: ''
})

// 搜索框闪烁状态
const isSearchBlinking = ref(false)
const searchInputRef = ref<HTMLInputElement | null>(null)
let blinkTimer: number | null = null

// 错题数据
const errors = ref([])

// 停止闪烁
const stopBlinking = () => {
  isSearchBlinking.value = false
  if (blinkTimer) {
    clearTimeout(blinkTimer)
    blinkTimer = null
  }
}

// 开始闪烁
const startBlinking = () => {
  isSearchBlinking.value = true
  // 3秒后自动停止闪烁
  if (blinkTimer) {
    clearTimeout(blinkTimer)
  }
  blinkTimer = window.setTimeout(() => {
    stopBlinking()
  }, 3000)
}

// 监听自定义事件（当用户已在当前页面时触发）
const handleTriggerBlink = () => {
  startBlinking()
}

// 从数据库获取错题数据
const fetchErrors = async () => {
  try {
    const questions = await getQuestions()
    const subjects = await getSubjects()
    
    // 转换数据格式
    errors.value = questions.map((question: any) => {
      const subject = subjects.find((s: any) => s.id === question.subject_id)
      return {
        id: question.id,
        subject: subject ? subject.name.toLowerCase() : 'unknown',
        subjectName: subject ? subject.name : '未知科目',
        difficulty: question.difficulty || 'medium',
        difficultyName: getDifficultyName(question.difficulty),
        content: question.question_content || '',
        date: formatDate(question.created_at),
        status: getStatus(question.review_status),
        statusText: getStatusText(question.review_status)
      }
    })
  } catch (error) {
    console.error('获取错题数据失败:', error)
    errors.value = []
  }
}

// 格式化日期
const formatDate = (dateString: string) => {
  if (!dateString) return ''
  const date = new Date(dateString)
  return date.toISOString().split('T')[0]
}

// 获取难度名称
const getDifficultyName = (difficulty: string) => {
  const difficultyMap: Record<string, string> = {
    'easy': '简单',
    'medium': '中等',
    'hard': '困难'
  }
  return difficultyMap[difficulty] || '中等'
}

// 获取状态
const getStatus = (reviewStatus: number) => {
  if (reviewStatus >= 3) return 'mastered'
  if (reviewStatus >= 1) return 'reviewed'
  return 'pending'
}

// 获取状态文本
const getStatusText = (reviewStatus: number) => {
  if (reviewStatus >= 3) return '已掌握'
  if (reviewStatus >= 1) return '已复习'
  return '待复习'
}

onMounted(() => {
  // 从其他页面跳转过来时检查是否需要闪烁
  if (route.query.focus === 'search') {
    startBlinking()
  }
  // 监听自定义事件
  window.addEventListener('trigger-search-blink', handleTriggerBlink)
  // 加载错题数据
  fetchErrors()
})

onUnmounted(() => {
  if (blinkTimer) {
    clearTimeout(blinkTimer)
  }
  window.removeEventListener('trigger-search-blink', handleTriggerBlink)
})

// 点击搜索框时停止闪烁
const onSearchFocus = () => {
  stopBlinking()
}

const filteredErrors = computed(() => {
  return errors.value.filter(error => {
    if (filters.value.subject && error.subject !== filters.value.subject) return false
    if (filters.value.status && error.status !== filters.value.status) return false
    if (filters.value.keyword && !error.content.includes(filters.value.keyword)) return false
    return true
  })
})

const viewError = (error: any) => {
  console.log('查看错题', error)
}
</script>

<style scoped>
.manage-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  max-width: 800px;
  margin: 0 auto;
}

.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.filter-select {
  flex: 1;
  min-width: 100px;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
}

.search-box {
  flex: 2;
  min-width: 150px;
}

.search-box input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  box-sizing: border-box;
}

.search-box input:focus {
  outline: none;
  border-color: var(--primary-color);
}

/* 搜索框闪烁动画 */
.search-box.blinking input {
  animation: searchBlink 0.6s ease-in-out infinite;
}

@keyframes searchBlink {
  0%, 100% {
    border-color: var(--border-color);
    box-shadow: 0 0 0 0 transparent;
  }
  50% {
    border-color: var(--primary-color);
    box-shadow: 0 0 8px rgba(25, 118, 210, 0.5);
  }
}

.error-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.error-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: all 0.3s;
}

.error-card:active {
  transform: scale(0.98);
}

.error-header {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.subject-tag {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.subject-tag.math {
  background: #e3f2fd;
  color: #1976d2;
}

.subject-tag.physics {
  background: #fff3e0;
  color: #e65100;
}

.subject-tag.chemistry {
  background: #f3e5f5;
  color: #7b1fa2;
}

.subject-tag.english {
  background: #e8f5e9;
  color: #43a047;
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

.error-content {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 12px;
  line-height: 1.5;
}

.error-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.error-date {
  color: var(--text-secondary);
}

.error-status {
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.error-status.pending {
  background: #fff3e0;
  color: #e65100;
}

.error-status.reviewed {
  background: #e3f2fd;
  color: #1976d2;
}

.error-status.mastered {
  background: #e8f5e9;
  color: #43a047;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-state p {
  font-size: 16px;
  margin: 0;
}
</style>