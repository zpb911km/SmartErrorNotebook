<template>
  <div class="manage-page">
    <div class="filter-bar">
      <!-- 自定义科目选择器 -->
      <div class="custom-subject-selector" 
           @mouseleave="hideKnowledgePanel"
           ref="subjectSelectorRef">
        <div 
          class="subject-select-trigger"
          @click="toggleSubjectDropdown"
          :class="{ open: showSubjectDropdown }"
        >
          <span class="selected-text">{{ selectedSubjectText }}</span>
          <span class="arrow">▼</span>
        </div>
        
        <!-- 科目下拉菜单 -->
        <div 
          v-show="showSubjectDropdown" 
          class="subject-dropdown"
        >
          <div 
            class="dropdown-item"
            :class="{ active: !filters.subject }"
            @click="selectSubject('')"
          >
            全部科目
          </div>
          <div 
            v-for="subject in userSubjects" 
            :key="subject.id" 
            class="dropdown-item"
            :class="{ active: filters.subject === subject.name }"
            @mouseenter="setCurrentHoverSubject(subject)"
            @click="selectSubject(subject.name)"
          >
            {{ subject.name }}
          </div>
        </div>
        
        <!-- 知识点悬浮面板 -->
        <div 
          v-if="showKnowledgeList && currentHoverSubject" 
          class="knowledge-panel"
          :style="knowledgePanelStyle"
        >
          <div class="knowledge-panel-header">
            <span class="subject-name">{{ currentHoverSubject.name }}的知识点</span>
            <button class="close-btn" @click="hideKnowledgePanel">×</button>
          </div>
          <div class="knowledge-list">
            <div 
              v-for="knowledge in subjectKnowledges" 
              :key="knowledge"
              class="knowledge-item"
              :class="{ active: filters.knowledge === knowledge }"
              @click="selectKnowledge(knowledge)"
            >
              {{ knowledge }}
            </div>
            <div v-if="subjectKnowledges.length === 0" class="no-knowledge">
              暂无知识点数据
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
          ref="searchInputRef"
          @focus="onSearchFocus"
          @click="onSearchFocus"
        >
      </div>
      
      <!-- 清除知识点筛选按钮 -->
      <button 
        v-if="filters.knowledge" 
        class="clear-knowledge-btn"
        @click="clearKnowledgeFilter"
      >
        清除知识点筛选
      </button>
    </div>

    <div class="error-list">
      <div v-for="error in filteredErrors" :key="error.id" class="error-card" @click="viewError(error)">
        <div class="error-header">
          <span class="subject-tag">{{ getSubjectName(error.subject_id) }}</span>
          <span class="difficulty-tag">{{ getDifficultyLevel(error.type) }}</span>
        </div>
        <div class="error-content">{{ error.prompt }}</div>
        <div class="error-source" v-if="getErrorSource(error.id)">
          知识点：{{ getErrorSource(error.id)?.knowledge }}
        </div>
        <div class="error-footer">
          <span class="error-date">{{ formatDate(error.id) }}</span>
          <span class="error-status">{{ getStatusText(error.id) }}</span>
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
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { testErrorQuestions, testSubjects, testSources, getDataUtils } from '../types/testdata'

const route = useRoute()

const filters = ref({
  subject: '',
  status: '',
  keyword: '',
  knowledge: '' // 新增知识点筛选
})

// 下拉菜单状态
const showSubjectDropdown = ref(false)
const selectedSubjectText = ref('全部科目')

// 悬浮面板相关状态
const showKnowledgeList = ref(false)
const currentHoverSubject = ref<any>(null)
const subjectSelectorRef = ref<HTMLElement | null>(null)
const knowledgePanelStyle = ref({})

// 搜索框闪烁状态
const isSearchBlinking = ref(false)
const searchInputRef = ref<HTMLInputElement | null>(null)
let blinkTimer: number | null = null

// 切换科目下拉菜单
const toggleSubjectDropdown = () => {
  showSubjectDropdown.value = !showSubjectDropdown.value
}

// 选择科目
const selectSubject = (subjectName: string) => {
  filters.value.subject = subjectName
  selectedSubjectText.value = subjectName || '全部科目'
  showSubjectDropdown.value = false
  // 如果选择了"全部科目"，清除知识点筛选
  if (!subjectName) {
    filters.value.knowledge = ''
  }
}

// 获取当前用户拥有的科目（基于错题数据）
const userSubjects = computed(() => {
  const subjectIds = [...new Set(testErrorQuestions.map(eq => eq.subject_id))]
  return testSubjects.filter(subject => subjectIds.includes(subject.id))
})

// 获取当前悬停科目的知识点列表
const subjectKnowledges = computed(() => {
  if (!currentHoverSubject.value) return []
  
  const subjectQuestions = testErrorQuestions.filter(
    eq => eq.subject_id === currentHoverSubject.value.id
  )
  
  const knowledgeSet = new Set<string>()
  subjectQuestions.forEach(question => {
    const source = getDataUtils.getSourceByQuestionId(question.id)
    if (source && source.knowledge) {
      knowledgeSet.add(source.knowledge)
    }
  })
  
  return Array.from(knowledgeSet).sort()
})

// 显示知识点面板
const showKnowledgePanel = async () => {
  await nextTick()
  if (subjectSelectorRef.value) {
    const rect = subjectSelectorRef.value.getBoundingClientRect()
    knowledgePanelStyle.value = {
      top: `${rect.top}px`,
      left: `${rect.right}px`,
      minWidth: '200px',
      zIndex: '1001'
    }
    showKnowledgeList.value = true
  }
}

// 隐藏知识点面板
const hideKnowledgePanel = () => {
  showKnowledgeList.value = false
  currentHoverSubject.value = null
}

// 设置当前悬停的科目
const setCurrentHoverSubject = (subject: any) => {
  currentHoverSubject.value = subject
  showKnowledgePanel()
}

// 选择知识点进行筛选
const selectKnowledge = (knowledge: string) => {
  filters.value.knowledge = knowledge
  hideKnowledgePanel()
  showSubjectDropdown.value = false
}

// 清除知识点筛选
const clearKnowledgeFilter = () => {
  filters.value.knowledge = ''
}

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

// 点击外部区域关闭下拉菜单
const handleClickOutside = (event: MouseEvent) => {
  if (subjectSelectorRef.value && !subjectSelectorRef.value.contains(event.target as Node)) {
    showSubjectDropdown.value = false
  }
}

onMounted(() => {
  // 从其他页面跳转过来时检查是否需要闪烁
  if (route.query.focus === 'search') {
    startBlinking()
  }
  // 监听自定义事件
  window.addEventListener('trigger-search-blink', handleTriggerBlink)
  // 监听点击外部事件
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  if (blinkTimer) {
    clearTimeout(blinkTimer)
  }
  window.removeEventListener('trigger-search-blink', handleTriggerBlink)
  document.removeEventListener('click', handleClickOutside)
})

// 点击搜索框时停止闪烁
const onSearchFocus = () => {
  stopBlinking()
}

const errors = ref(testErrorQuestions)

const filteredErrors = computed(() => {
  return errors.value.filter((error: any) => {
    // 科目过滤 - 使用科目ID映射
    if (filters.value.subject && getSubjectName(error.subject_id) !== filters.value.subject) return false
    
    // 知识点过滤
    if (filters.value.knowledge) {
      const source = getDataUtils.getSourceByQuestionId(error.id)
      if (!source || source.knowledge !== filters.value.knowledge) return false
    }
    
    // 状态过滤 - 这里需要根据实际情况调整
    if (filters.value.status) {
      const statusText = getStatusText(error.id)
      if (statusText !== filters.value.status) return false
    }
    
    // 关键词过滤 - 搜索题干内容
    if (filters.value.keyword && !error.prompt.includes(filters.value.keyword)) return false
    
    return true
  })
})

// 辅助函数：根据科目ID获取科目名称
const getSubjectName = (subjectId: string): string => {
  const subject = getDataUtils.getSubjectById(subjectId)
  return subject ? subject.name : '未知科目'
}

// 辅助函数：获取错题的来源信息
const getErrorSource = (questionId: string) => {
  return getDataUtils.getSourceByQuestionId(questionId)
}

// 辅助函数：根据题型获取难度等级
const getDifficultyLevel = (type: string): string => {
  const difficultyMap: Record<string, string> = {
    '计算题': '中等',
    '翻译题': '简单',
    '简答题': '中等'
  }
  return difficultyMap[type] || '中等'
}

// 辅助函数：格式化日期显示
const formatDate = (id: string): string => {
  // 根据ID生成相对日期
  const idNum = parseInt(id.split('_')[1])
  const daysAgo = idNum - 1
  const date = new Date()
  date.setDate(date.getDate() - daysAgo)
  return date.toLocaleDateString('zh-CN')
}

// 辅助函数：获取状态文本
const getStatusText = (id: string): string => {
  const idNum = parseInt(id.split('_')[1])
  const statuses = ['待复习', '已复习', '已掌握', '待复习']
  return statuses[(idNum - 1) % statuses.length]
}

// 查看错题详情
const viewError = (error: any) => {
  console.log('查看错题:', error)
  // 这里可以跳转到错题详情页面
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
  position: relative;
}

.custom-subject-selector {
  position: relative;
  flex: 1;
  min-width: 120px;
}

.subject-select-trigger {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.subject-select-trigger:hover {
  border-color: var(--primary-color);
}

.subject-select-trigger.open {
  border-color: var(--primary-color);
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
}

.selected-text {
  flex: 1;
}

.arrow {
  transition: transform 0.2s;
  font-size: 12px;
}

.subject-select-trigger.open .arrow {
  transform: rotate(180deg);
}

.subject-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-top: none;
  border-radius: 0 0 8px 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 100;
  max-height: 200px;
  overflow-y: auto;
}

.dropdown-item {
  padding: 10px 12px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-primary);
  transition: all 0.2s;
  border-left: 3px solid transparent;
}

.dropdown-item:hover {
  background: var(--hover-bg);
  border-left-color: var(--primary-color);
}

.dropdown-item.active {
  background: var(--primary-light);
  border-left-color: var(--primary-color);
  font-weight: 500;
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
  cursor: pointer;
}

.filter-select:hover {
  border-color: var(--primary-color);
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

/* 知识点悬浮面板样式 */
.knowledge-panel {
  position: fixed;
  z-index: 1000;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-height: 300px;
  overflow-y: auto;
}

.knowledge-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--hover-bg);
}

.subject-name {
  font-weight: 500;
  color: var(--text-primary);
  font-size: 14px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.close-btn:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

.knowledge-list {
  padding: 8px 0;
}

.knowledge-item {
  padding: 10px 16px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  transition: all 0.2s;
  border-left: 3px solid transparent;
}

.knowledge-item:hover {
  background: var(--hover-bg);
  border-left-color: var(--primary-color);
}

.knowledge-item.active {
  background: var(--primary-light);
  border-left-color: var(--primary-color);
  font-weight: 500;
}

.no-knowledge {
  padding: 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.clear-knowledge-btn {
  padding: 8px 12px;
  background: var(--warning-color);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.2s;
}

.clear-knowledge-btn:hover {
  background: var(--warning-dark);
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
  margin-bottom: 8px;
  line-height: 1.5;
}

.error-source {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 12px;
  padding: 6px 8px;
  background: var(--hover-bg);
  border-radius: 4px;
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