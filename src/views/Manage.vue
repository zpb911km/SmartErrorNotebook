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

// 停止闪烁
const stopBlinking = () => {
  isSearchBlinking.value = false
  if (blinkTimer) {
    clearTimeout(blinkTimer)
    blinkTimer = null
  }
}

// 检查是否需要闪烁
onMounted(() => {
  if (route.query.focus === 'search') {
    isSearchBlinking.value = true
    // 3秒后自动停止闪烁
    blinkTimer = window.setTimeout(() => {
      stopBlinking()
    }, 3000)
  }
})

onUnmounted(() => {
  if (blinkTimer) {
    clearTimeout(blinkTimer)
  }
})

// 点击搜索框时停止闪烁
const onSearchFocus = () => {
  stopBlinking()
}

const errors = ref([
  {
    id: 1,
    subject: 'math',
    subjectName: '数学',
    difficulty: 'medium',
    difficultyName: '中等',
    content: '求函数 f(x) = x² - 2x + 1 的最小值',
    date: '2026-01-15',
    status: 'pending',
    statusText: '待复习'
  },
  {
    id: 2,
    subject: 'physics',
    subjectName: '物理',
    difficulty: 'hard',
    difficultyName: '困难',
    content: '一个质量为 2kg 的物体从 10m 高处自由落下，求落地时的速度',
    date: '2026-01-14',
    status: 'reviewed',
    statusText: '已复习'
  },
  {
    id: 3,
    subject: 'english',
    subjectName: '英语',
    difficulty: 'easy',
    difficultyName: '简单',
    content: '翻译句子：The quick brown fox jumps over the lazy dog',
    date: '2026-01-13',
    status: 'mastered',
    statusText: '已掌握'
  },
  {
    id: 4,
    subject: 'chemistry',
    subjectName: '化学',
    difficulty: 'medium',
    difficultyName: '中等',
    content: '写出硫酸的化学式并说明其性质',
    date: '2026-01-12',
    status: 'pending',
    statusText: '待复习'
  }
])

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
  padding: 20px;
  padding-bottom: 80px;
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