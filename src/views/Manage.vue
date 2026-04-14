<template>
  <div class="manage-page">
    <div class="filter-bar">
      <!-- 科目筛选 - 向右展开的级联菜单 -->
      <div 
        class="filter-select-wrapper"
      >
        <!-- 自定义下拉框 -->
        <div 
          class="custom-select"
          @click="toggleSubjectDropdown"
          @mouseenter="showSubjectList"
        >
          <div class="custom-select-value">
            {{ selectedSubjectName || '全部科目' }}
          </div>
          <span class="custom-select-arrow" :class="{ 'rotated': subjectDropdownVisible }">▼</span>
        </div>
        
        <!-- 清除筛选按钮 -->
        <button 
          v-if="filters.subject_id" 
          class="clear-filter-btn"
          @click="clearSubjectFilter"
          title="清除科目筛选"
        >
          ×
        </button>
        
        <!-- 向下展开的级联窗口 -->
        <div v-if="subjectDropdownVisible || cascadeVisible" class="cascade-popup">
          <!-- 科目列 -->
          <div class="cascade-column">
            <div class="column-title">科目</div>
            <div class="column-items">
              <div 
                v-for="subject in subjects" 
                :key="subject.id"
                class="cascade-item"
                :class="{ active: filters.subject_id === subject.id }"
                @mouseenter="showCascadeMenuForSubject(subject.id)"
                @click="selectSubject(subject.id)"
              >
                {{ subject.name }}
                <span class="arrow-indicator">›</span>
              </div>
            </div>
          </div>
          
          <!-- 书名列 -->
          <div v-if="currentSubjectId && books.length > 0" class="cascade-column">
            <div class="column-title">书名</div>
            <div class="column-items">
              <div 
                v-for="book in books" 
                :key="book"
                class="cascade-item"
                :class="{ active: filters.book === book }"
                @mouseenter="showChapters(book)"
                @click="selectBook(book)"
              >
                {{ book || '未分类' }}
                <span class="arrow-indicator">›</span>
              </div>
            </div>
          </div>
          
          <!-- 章节列 -->
          <div v-if="currentBook && chapters.length > 0" class="cascade-column">
            <div class="column-title">章节</div>
            <div class="column-items">
              <div 
                v-for="chapter in chapters" 
                :key="chapter"
                class="cascade-item"
                :class="{ active: filters.chapter === chapter }"
                @mouseenter="showKnowledges(chapter)"
                @click="selectChapter(chapter)"
              >
                {{ chapter || '未分类' }}
                <span class="arrow-indicator">›</span>
              </div>
            </div>
          </div>
          
          <!-- 知识点列 -->
          <div v-if="currentChapter && knowledges.length > 0" class="cascade-column">
            <div class="column-title">知识点</div>
            <div class="column-items">
              <div 
                v-for="knowledge in knowledges" 
                :key="knowledge"
                class="cascade-item"
                :class="{ active: filters.knowledge === knowledge }"
                @click="selectKnowledge(knowledge)"
              >
                {{ knowledge || '未分类' }}
              </div>
            </div>
          </div>
        </div>
      </div>

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
          @focus="onSearchFocus"
          @click="onSearchFocus"
        >
      </div>
    </div>

    <div class="error-list">
      <div v-for="error in filteredErrors" :key="error.id" class="error-card" @click="viewError(error)">
        <div class="error-header">
          <span class="subject-tag" :style="getSubjectStyle(error.subject_id)">{{ error.subjectName }}</span>
          <span class="difficulty-tag" :class="getDifficultyClass(error.difficulty)">{{ error.difficultyName }}</span>
        </div>
        <div class="error-content">{{ truncateContent(error.content, 100) }}</div>
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
import { useRouter, useRoute } from 'vue-router'
import { getQuestions } from '../apis/errorQuestions'
import { getSubjects } from '../apis/subjects'
import { getBooks, getChapters, getKnowledges } from '../apis/sources'
import type { Subject } from '../types'

const router = useRouter()
const route = useRoute()

const filters = ref({
  subject_id: '',
  book: '',
  chapter: '',
  knowledge: '',
  status: '',
  keyword: ''
})

// 搜索框闪烁状态
const isSearchBlinking = ref(false)
let blinkTimer: number | null = null

// 数据列表 - 使用 any 类型以兼容后端返回的额外字段
const errors = ref<any[]>([])
const subjects = ref<Subject[]>([])

// 级联菜单状态
const cascadeVisible = ref(false)
const currentSubjectId = ref<string | null>(null)
const currentBook = ref<string | null>(null)
const currentChapter = ref<string | null>(null)
const books = ref<string[]>([])
const chapters = ref<string[]>([])
const knowledges = ref<string[]>([])
let hideTimer: number | null = null

// 科目下拉框状态
const subjectDropdownVisible = ref(false)

// 计算选中的科目名称
const selectedSubjectName = computed(() => {
  if (!filters.value.subject_id) return ''
  const subject = subjects.value.find(s => s.id === filters.value.subject_id)
  return subject?.name || ''
})

// 停止闪烁
// 切换科目下拉框
const toggleSubjectDropdown = () => {
  subjectDropdownVisible.value = !subjectDropdownVisible.value
  // 如果关闭下拉框，也隐藏级联菜单
  if (!subjectDropdownVisible.value) {
    hideAllMenus()
  }
}

// 显示科目列表
const showSubjectList = async () => {
  subjectDropdownVisible.value = true
  // 加载所有科目的书名（如果没有选中具体科目）
  if (!filters.value.subject_id) {
    books.value = []
    chapters.value = []
    knowledges.value = []
  }
}

// 选择科目
const selectSubject = (subjectId: string) => {
  filters.value.subject_id = subjectId
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  
  if (subjectId) {
    showCascadeMenuForSubject(subjectId)
  } else {
    hideAllMenus()
  }
}

// 清除科目筛选
const clearSubjectFilter = () => {
  filters.value.subject_id = ''
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  currentSubjectId.value = null
  currentBook.value = null
  currentChapter.value = null
  books.value = []
  chapters.value = []
  knowledges.value = []
  subjectDropdownVisible.value = false
  cascadeVisible.value = false
  fetchData()
}

// 为指定科目显示级联菜单
const showCascadeMenuForSubject = async (subjectId: string) => {
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
  cascadeVisible.value = true
  currentSubjectId.value = subjectId
  currentBook.value = null
  currentChapter.value = null
  chapters.value = []
  knowledges.value = []
  
  // 加载当前科目的书名
  try {
    books.value = await getBooks(subjectId)
  } catch (error) {
    console.error('获取书名失败:', error)
    books.value = []
  }
}

// 隐藏所有菜单
const hideAllMenus = () => {
  hideTimer = window.setTimeout(() => {
    subjectDropdownVisible.value = false
    cascadeVisible.value = false
    currentSubjectId.value = null
    currentBook.value = null
    currentChapter.value = null
    chapters.value = []
    knowledges.value = []
  }, 200)
}

// 隐藏级联菜单
const hideCascadeMenu = () => {
  hideTimer = window.setTimeout(() => {
    cascadeVisible.value = false
    currentBook.value = null
    currentChapter.value = null
    chapters.value = []
    knowledges.value = []
  }, 200)
}

// 显示章节
const showChapters = async (book: string) => {
  currentBook.value = book
  currentChapter.value = null
  knowledges.value = []
  
  if (filters.value.subject_id && book) {
    try {
      chapters.value = await getChapters(book, filters.value.subject_id)
    } catch (error) {
      console.error('获取章节失败:', error)
      chapters.value = []
    }
  } else {
    chapters.value = []
  }
}

// 显示知识点
const showKnowledges = async (chapter: string) => {
  currentChapter.value = chapter
  
  if (filters.value.subject_id && currentBook.value && chapter) {
    try {
      knowledges.value = await getKnowledges(currentBook.value, chapter, filters.value.subject_id)
    } catch (error) {
      console.error('获取知识点失败:', error)
      knowledges.value = []
    }
  } else {
    knowledges.value = []
  }
}

// 选择书名
const selectBook = (book: string) => {
  filters.value.book = book
  filters.value.chapter = ''
  filters.value.knowledge = ''
  hideCascadeMenu()
}

// 选择章节
const selectChapter = (chapter: string) => {
  filters.value.chapter = chapter
  filters.value.knowledge = ''
  hideCascadeMenu()
}

// 选择知识点
const selectKnowledge = (knowledge: string) => {
  filters.value.knowledge = knowledge
  hideCascadeMenu()
}

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

// 从数据库获取数据
const fetchData = async () => {
  try {
    // 并行获取科目和错题数据
    const [subjectsData, questionsData] = await Promise.all([
      getSubjects(),
      getQuestions()
    ])
    
    subjects.value = subjectsData
    // 后端返回的数据包含 created_at 和 updated_at 等额外字段
    errors.value = questionsData as any[]
  } catch (error) {
    console.error('获取数据失败:', error)
    errors.value = []
    subjects.value = []
  }
}

// 格式化日期（后端返回的是秒级时间戳）
const formatDate = (timestamp: number) => {
  if (!timestamp) return ''
  // 后端返回的是秒级时间戳，需要转换为毫秒
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN')
}

// 截断内容
const truncateContent = (content: string, maxLength: number) => {
  if (!content) return ''
  if (content.length <= maxLength) return content
  return content.substring(0, maxLength) + '...'
}

// 获取难度等级（基于 SRS 数据）
const getDifficultyLevel = (_questionId: string): number => {
  // TODO: 从 SRS 数据中获取实际难度
  // 目前返回默认值 2（中等）
  return 2
}

// 获取难度名称
const getDifficultyName = (level: number) => {
  if (level <= 1) return '简单'
  if (level <= 2) return '中等'
  return '困难'
}

// 获取难度样式类
const getDifficultyClass = (level: number) => {
  if (level <= 1) return 'easy'
  if (level <= 2) return 'medium'
  return 'hard'
}

// 获取复习状态（基于 SRS 数据）
const getReviewStatus = (_questionId: string) => {
  // TODO: 从 SRS 数据中获取实际状态
  // 目前返回默认值
  return {
    status: 'pending',
    statusText: '待复习'
  }
}

// 获取科目样式
const getSubjectStyle = (subjectId: string) => {
  const subject = subjects.value.find(s => s.id === subjectId)
  if (subject?.color) {
    return {
      backgroundColor: `${subject.color}20`, // 添加透明度
      color: subject.color
    }
  }
  // 默认样式
  return {
    backgroundColor: '#e3f2fd',
    color: '#1976d2'
  }
}

onMounted(() => {
  // 从其他页面跳转过来时检查是否需要闪烁
  if (route.query.focus === 'search') {
    startBlinking()
  }
  // 监听自定义事件
  window.addEventListener('trigger-search-blink', handleTriggerBlink)
  // 加载数据
  fetchData()
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

// 过滤后的错题列表
const filteredErrors = computed(() => {
  return errors.value
    .map((question: any) => {
      const subject = subjects.value.find(s => s.id === question.subjectid)
      const difficulty = getDifficultyLevel(question.id)
      const { status, statusText } = getReviewStatus(question.id)
      
      return {
        id: question.id,
        subject_id: question.subjectid,
        source_id: question.sourceid,
        subjectName: subject?.name || '未知科目',
        difficulty,
        difficultyName: getDifficultyName(difficulty),
        content: question.prompt || '',
        // 后端返回的是秒级时间戳
        date: formatDate(question.updated_at || question.created_at || 0),
        status,
        statusText
      }
    })
    .filter(error => {
      // 科目筛选
      if (filters.value.subject_id && error.subject_id !== filters.value.subject_id) {
        return false
      }
      // 状态筛选
      if (filters.value.status && error.status !== filters.value.status) {
        return false
      }
      // 关键词搜索
      if (filters.value.keyword) {
        const keyword = filters.value.keyword.toLowerCase()
        const content = error.content.toLowerCase()
        if (!content.includes(keyword)) {
          return false
        }
      }
      return true
    })
})

// 查看错题详情
const viewError = (error: any) => {
  router.push({
    name: 'Preview',
    params: { id: error.id }
  })
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
  align-items: flex-start;
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

/* 级联筛选器容器 */
.filter-select-wrapper {
  position: relative;
  flex: 1;
  min-width: 100px;
}

/* 向下展开的级联弹窗 */
.cascade-popup {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  display: flex;
  gap: 0;
  z-index: 1000;
  min-width: 600px;
  max-width: 800px;
  max-height: 500px;
  animation: cascadeSlideDown 0.2s ease-out;
  overflow: hidden;
}

@keyframes cascadeSlideDown {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
.custom-select {
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: all 0.2s;
}

.custom-select:hover {
  border-color: var(--primary-color);
}

.custom-select-value {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-select-arrow {
  font-size: 10px;
  color: var(--text-secondary);
  margin-left: 8px;
  transition: transform 0.2s;
}

.custom-select-arrow.rotated {
  transform: rotate(180deg);
}

/* 清除筛选按钮 */
.clear-filter-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--text-secondary);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  padding: 0;
  margin-left: 4px;
}

.clear-filter-btn:hover {
  background: #ff4d4f;
  border-color: #ff4d4f;
  color: white;
}

/* 级联列 */
.cascade-column {
  flex: 1;
  min-width: 150px;
  max-width: 200px;
  border-right: 1px solid var(--border-color);
  padding: 8px 0;
  max-height: 500px;
  overflow-y: auto;
}

.cascade-column:last-child {
  border-right: none;
}

/* 列标题 */
.column-title {
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 4px;
  background: rgba(0, 0, 0, 0.02);
}

/* 级联项目列表 */
.cascade-items {
  max-height: 460px;
  overflow-y: auto;
}

/* 级联项目 */
.cascade-item {
  padding: 10px 12px;
  font-size: 14px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.cascade-item:hover {
  background: var(--primary-color);
  color: white;
}

.cascade-item.active {
  background: var(--primary-color);
  color: white;
  font-weight: 500;
}

.cascade-item:active {
  transform: scale(0.98);
}

/* 箭头指示符 */
.arrow-indicator {
  margin-left: 8px;
  color: var(--text-secondary);
  font-size: 18px;
  line-height: 1;
  flex-shrink: 0;
}

.cascade-item:hover .arrow-indicator,
.cascade-item.active .arrow-indicator {
  color: white;
}

/* 动画 */
@keyframes cascadeFadeIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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
  white-space: nowrap;
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