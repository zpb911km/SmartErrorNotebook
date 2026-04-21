<template>
  <div class="manage-page">
    <!-- 搜索栏 - 第一行 -->
    <div class="search-bar">
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

    <!-- 筛选栏 - 第二行 -->
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
        
        <!-- 向下展开的级联窗口 -->
        <div v-if="subjectDropdownVisible || cascadeVisible" class="cascade-popup">
          <!-- 关闭按钮 -->
          <button 
            class="cascade-close-btn"
            @click="closeCascadeWindow"
            title="关闭"
          >
            ×
          </button>
          
          <!-- 科目列 -->
          <div class="cascade-column">
            <div class="column-title">科目</div>
            <div class="column-items">
              <div 
                v-for="subject in subjects" 
                :key="subject.id"
                class="cascade-item"
                :class="{ active: filters.subject_id === subject.id }"
                @mouseenter="!hasClicked && showCascadeMenuForSubject(subject.id)"
                @click="handleSubjectClick(subject.id)"
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
                @mouseenter="!hasBookClicked && showChapters(book)"
                @click="handleBookClick(book)"
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
                @mouseenter="!hasChapterClicked && showKnowledges(chapter)"
                @click="handleChapterClick(chapter)"
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

      <!-- 状态筛选 -->
      <div class="filter-select-wrapper">
        <div 
          class="custom-select"
          @click="toggleStatusDropdown"
        >
          <div class="custom-select-value">
            {{ statusText || '全部状态' }}
          </div>
          <span class="custom-select-arrow" :class="{ 'rotated': statusDropdownVisible }">▼</span>
        </div>
        
        <div v-if="statusDropdownVisible" class="dropdown-popup">
          <div 
            class="dropdown-item"
            :class="{ active: filters.status === '' }"
            @click="selectStatus('')"
          >
            全部状态
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.status === 'pending' }"
            @click="selectStatus('pending')"
          >
            待复习
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.status === 'reviewed' }"
            @click="selectStatus('reviewed')"
          >
            已复习
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.status === 'mastered' }"
            @click="selectStatus('mastered')"
          >
            已掌握
          </div>
        </div>
      </div>

      <!-- 难度筛选 -->
      <div class="filter-select-wrapper">
        <div 
          class="custom-select"
          @click="toggleDifficultyDropdown"
        >
          <div class="custom-select-value">
            {{ difficultyText || '全部难度' }}
          </div>
          <span class="custom-select-arrow" :class="{ 'rotated': difficultyDropdownVisible }">▼</span>
        </div>
        
        <div v-if="difficultyDropdownVisible" class="dropdown-popup">
          <div 
            class="dropdown-item"
            :class="{ active: filters.difficulty === '' }"
            @click="selectDifficulty('')"
          >
            全部难度
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.difficulty === 'easy' }"
            @click="selectDifficulty('easy')"
          >
            简单
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.difficulty === 'medium' }"
            @click="selectDifficulty('medium')"
          >
            中等
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.difficulty === 'hard' }"
            @click="selectDifficulty('hard')"
          >
            困难
          </div>
        </div>
      </div>

      <!-- 掌握程度筛选 -->
      <div class="filter-select-wrapper">
        <div 
          class="custom-select"
          @click="toggleMasteryDropdown"
        >
          <div class="custom-select-value">
            {{ masteryText || '全部掌握程度' }}
          </div>
          <span class="custom-select-arrow" :class="{ 'rotated': masteryDropdownVisible }">▼</span>
        </div>
        
        <div v-if="masteryDropdownVisible" class="dropdown-popup">
          <div 
            class="dropdown-item"
            :class="{ active: filters.mastery === '' }"
            @click="selectMastery('')"
          >
            全部掌握程度
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.mastery === 'unmastered' }"
            @click="selectMastery('unmastered')"
          >
            未掌握 (0-30%)
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.mastery === 'partial' }"
            @click="selectMastery('partial')"
          >
            部分掌握 (30-70%)
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.mastery === 'mastered' }"
            @click="selectMastery('mastered')"
          >
            已掌握 (70-100%)
          </div>
        </div>
      </div>

      <!-- 时间范围筛选 -->
      <div class="filter-select-wrapper">
        <div 
          class="custom-select"
          @click="toggleDateRangeDropdown"
        >
          <div class="custom-select-value">
            {{ dateRangeText || '全部时间' }}
          </div>
          <span class="custom-select-arrow" :class="{ 'rotated': dateRangeDropdownVisible }">▼</span>
        </div>
        
        <div v-if="dateRangeDropdownVisible" class="dropdown-popup">
          <div 
            class="dropdown-item"
            :class="{ active: filters.date_range === 'all' }"
            @click="selectDateRange('all')"
          >
            全部时间
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.date_range === '7days' }"
            @click="selectDateRange('7days')"
          >
            最近7天
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.date_range === '30days' }"
            @click="selectDateRange('30days')"
          >
            最近30天
          </div>
          <div 
            class="dropdown-item"
            :class="{ active: filters.date_range === '90days' }"
            @click="selectDateRange('90days')"
          >
            最近90天
          </div>
        </div>
      </div>

      <!-- 标签筛选 -->
      <div class="filter-select-wrapper">
        <div 
          class="custom-select"
          @click="toggleTagDropdown"
        >
          <div class="custom-select-value">
            {{ filters.tag || '全部标签' }}
          </div>
          <span class="custom-select-arrow" :class="{ 'rotated': tagDropdownVisible }">▼</span>
        </div>
        
        <div v-if="tagDropdownVisible" class="dropdown-popup tag-dropdown">
          <div 
            class="dropdown-item"
            :class="{ active: filters.tag === '' }"
            @click="selectTag('')"
          >
            全部标签
          </div>
          <div 
            v-for="tag in availableTags" 
            :key="tag"
            class="dropdown-item"
            :class="{ active: filters.tag === tag }"
            @click="selectTag(tag)"
          >
            {{ tag }}
          </div>
        </div>
      </div>
    </div>

    <!-- 已选筛选条件 -->
    <div v-if="activeFilters.length > 0" class="active-filters">
      <span class="active-filters-label">已选筛选：</span>
      <span v-for="filter in activeFilters" :key="filter.key" class="filter-tag">
        {{ filter.label }}
        <button @click="removeFilter(filter.key)" class="filter-tag-close">×</button>
      </span>
      <button @click="clearAllFilters" class="clear-all-btn">清除所有</button>
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
import { getErrorTags } from '../apis/errorTags'
import type { Subject } from '../types'

const router = useRouter()
const route = useRoute()

// 本地筛选状态
const filters = ref({
  subject_id: '',
  book: '',
  chapter: '',
  knowledge: '',
  status: '',
  keyword: '',
  difficulty: '',       // 'easy' | 'medium' | 'hard'
  mastery: '',          // 'unmastered' | 'partial' | 'mastered'
  date_range: 'all',    // '7days' | '30days' | '90days' | 'all'
  tag: ''               // 标签名称
})

// 搜索框闪烁状态
const isSearchBlinking = ref(false)
let blinkTimer: number | null = null

// 数据列表 - 使用 any 类型以兼容后端返回的额外字段
const errors = ref<any[]>([])
const subjects = ref<Subject[]>([])
const availableTags = ref<string[]>([])

// 错题和标签的映射关系（question_id -> 标签名称数组）
const questionTagsMap = ref<Map<string, string[]>>(new Map())

// 筛选器引用
const statusSelectRef = ref<HTMLSelectElement>()
const difficultySelectRef = ref<HTMLSelectElement>()
const masterySelectRef = ref<HTMLSelectElement>()
const dateRangeSelectRef = ref<HTMLSelectElement>()
const tagSelectRef = ref<HTMLSelectElement>()

// 级联菜单状态
const cascadeVisible = ref(false)
const currentSubjectId = ref<string | null>(null)
const currentBook = ref<string | null>(null)
const currentChapter = ref<string | null>(null)
const books = ref<string[]>([])
const chapters = ref<string[]>([])
const knowledges = ref<string[]>([])
let hideTimer: number | null = null

// 点击状态标记 - 记录用户是否已经点击过某项
const hasClicked = ref(false)
const hasBookClicked = ref(false)
const hasChapterClicked = ref(false)

// 科目下拉框状态
const subjectDropdownVisible = ref(false)

// 其他筛选器下拉框状态
const statusDropdownVisible = ref(false)
const difficultyDropdownVisible = ref(false)
const masteryDropdownVisible = ref(false)
const dateRangeDropdownVisible = ref(false)
const tagDropdownVisible = ref(false)

// 计算选中的科目名称
const selectedSubjectName = computed(() => {
  if (!filters.value.subject_id) return ''
  const subject = subjects.value.find(s => s.id === filters.value.subject_id)
  return subject?.name || ''
})

// 计算状态文本
const statusText = computed(() => {
  if (!filters.value.status) return ''
  const statusMap: Record<string, string> = {
    pending: '待复习',
    reviewed: '已复习',
    mastered: '已掌握'
  }
  return statusMap[filters.value.status] || ''
})

// 计算难度文本
const difficultyText = computed(() => {
  if (!filters.value.difficulty) return ''
  const difficultyMap: Record<string, string> = {
    easy: '简单',
    medium: '中等',
    hard: '困难'
  }
  return difficultyMap[filters.value.difficulty] || ''
})

// 计算掌握程度文本
const masteryText = computed(() => {
  if (!filters.value.mastery) return ''
  const masteryMap: Record<string, string> = {
    unmastered: '未掌握 (0-30%)',
    partial: '部分掌握 (30-70%)',
    mastered: '已掌握 (70-100%)'
  }
  return masteryMap[filters.value.mastery] || ''
})

// 计算时间范围文本
const dateRangeText = computed(() => {
  if (!filters.value.date_range || filters.value.date_range === 'all') return ''
  const dateRangeMap: Record<string, string> = {
    '7days': '最近7天',
    '30days': '最近30天',
    '90days': '最近90天'
  }
  return dateRangeMap[filters.value.date_range] || ''
})

// 停止闪烁
// 切换科目下拉框
const toggleSubjectDropdown = () => {
  subjectDropdownVisible.value = !subjectDropdownVisible.value
  // 如果关闭下拉框，也隐藏级联菜单
  if (!subjectDropdownVisible.value) {
    hideAllMenus()
  }
  // 关闭其他下拉框
  closeOtherDropdowns('subject')
}

// 切换状态下拉框
const toggleStatusDropdown = () => {
  statusDropdownVisible.value = !statusDropdownVisible.value
  closeOtherDropdowns('status')
}

// 切换难度下拉框
const toggleDifficultyDropdown = () => {
  difficultyDropdownVisible.value = !difficultyDropdownVisible.value
  closeOtherDropdowns('difficulty')
}

// 切换掌握程度下拉框
const toggleMasteryDropdown = () => {
  masteryDropdownVisible.value = !masteryDropdownVisible.value
  closeOtherDropdowns('mastery')
}

// 切换时间范围下拉框
const toggleDateRangeDropdown = () => {
  dateRangeDropdownVisible.value = !dateRangeDropdownVisible.value
  closeOtherDropdowns('dateRange')
}

// 切换标签下拉框
const toggleTagDropdown = () => {
  tagDropdownVisible.value = !tagDropdownVisible.value
  closeOtherDropdowns('tag')
}

// 关闭其他下拉框
const closeOtherDropdowns = (current: string) => {
  if (current !== 'subject') subjectDropdownVisible.value = false
  if (current !== 'status') statusDropdownVisible.value = false
  if (current !== 'difficulty') difficultyDropdownVisible.value = false
  if (current !== 'mastery') masteryDropdownVisible.value = false
  if (current !== 'dateRange') dateRangeDropdownVisible.value = false
  if (current !== 'tag') tagDropdownVisible.value = false
}

// 选择状态
const selectStatus = (status: string) => {
  filters.value.status = status
  statusDropdownVisible.value = false
}

// 选择难度
const selectDifficulty = (difficulty: string) => {
  filters.value.difficulty = difficulty
  difficultyDropdownVisible.value = false
}

// 选择掌握程度
const selectMastery = (mastery: string) => {
  filters.value.mastery = mastery
  masteryDropdownVisible.value = false
}

// 选择时间范围
const selectDateRange = (dateRange: string) => {
  filters.value.date_range = dateRange
  dateRangeDropdownVisible.value = false
}

// 选择标签
const selectTag = (tag: string) => {
  filters.value.tag = tag
  tagDropdownVisible.value = false
}

// 显示科目列表
const showSubjectList = async () => {
  // 关闭其他筛选器
  closeAllOtherFilters('subject')
  
  subjectDropdownVisible.value = true
  // 加载所有科目的书名（如果没有选中具体科目）
  if (!filters.value.subject_id) {
    books.value = []
    chapters.value = []
    knowledges.value = []
  }
}

// 状态筛选获得焦点时关闭其他筛选
const onStatusFocus = () => {
  closeAllOtherFilters('status')
}

// 难度筛选获得焦点时关闭其他筛选
const onDifficultyFocus = () => {
  closeAllOtherFilters('difficulty')
}

// 掌握程度筛选获得焦点时关闭其他筛选
const onMasteryFocus = () => {
  closeAllOtherFilters('mastery')
}

// 时间范围筛选获得焦点时关闭其他筛选
const onDateRangeFocus = () => {
  closeAllOtherFilters('dateRange')
}

// 标签筛选获得焦点时关闭其他筛选
const onTagFocus = () => {
  closeAllOtherFilters('tag')
}

// 关闭其他筛选器的通用函数
const closeAllOtherFilters = (current: string) => {
  if (current !== 'subject' && subjectDropdownVisible.value) {
    subjectDropdownVisible.value = false
    cascadeVisible.value = false
    hasClicked.value = false
    hasBookClicked.value = false
    hasChapterClicked.value = false
  }
  if (current !== 'status' && statusSelectRef.value) {
    statusSelectRef.value.blur()
  }
  if (current !== 'difficulty' && difficultySelectRef.value) {
    difficultySelectRef.value.blur()
  }
  if (current !== 'mastery' && masterySelectRef.value) {
    masterySelectRef.value.blur()
  }
  if (current !== 'dateRange' && dateRangeSelectRef.value) {
    dateRangeSelectRef.value.blur()
  }
  if (current !== 'tag' && tagSelectRef.value) {
    tagSelectRef.value.blur()
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

// 处理科目点击 - 设置点击状态并选择科目
const handleSubjectClick = (subjectId: string) => {
  hasClicked.value = true
  selectSubject(subjectId)
}

// 处理书名点击 - 设置点击状态并选择书名
const handleBookClick = async (book: string) => {
  hasBookClicked.value = true
  hasChapterClicked.value = false
  
  // 直接更新书名列数据和下级数据
  currentBook.value = book
  currentChapter.value = null
  chapters.value = []
  knowledges.value = []
  
  // 加载该书名的章节
  if (filters.value.subject_id && book) {
    try {
      chapters.value = await getChapters(book, filters.value.subject_id)
    } catch (error) {
      console.error('获取章节失败:', error)
      chapters.value = []
    }
  }
  
  selectBook(book)
}

// 处理章节点击 - 设置点击状态并选择章节
const handleChapterClick = (chapter: string) => {
  hasChapterClicked.value = true
  selectChapter(chapter)
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
    // 重置点击状态
    hasClicked.value = false
    hasBookClicked.value = false
    hasChapterClicked.value = false
  }, 200)
}

// 手动关闭级联窗口
const closeCascadeWindow = () => {
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
  subjectDropdownVisible.value = false
  cascadeVisible.value = false
  currentSubjectId.value = null
  currentBook.value = null
  currentChapter.value = null
  books.value = []
  chapters.value = []
  knowledges.value = []
  // 重置点击状态
  hasClicked.value = false
  hasBookClicked.value = false
  hasChapterClicked.value = false
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
  // 不隐藏菜单，保持打开状态
}

// 选择章节
const selectChapter = (chapter: string) => {
  filters.value.chapter = chapter
  filters.value.knowledge = ''
  // 不隐藏菜单，保持打开状态
}

// 选择知识点
const selectKnowledge = (knowledge: string) => {
  filters.value.knowledge = knowledge
  // 关闭所有菜单
  subjectDropdownVisible.value = false
  cascadeVisible.value = false
  currentSubjectId.value = null
  currentBook.value = null
  currentChapter.value = null
  books.value = []
  chapters.value = []
  knowledges.value = []
  // 重置点击状态
  hasClicked.value = false
  hasBookClicked.value = false
  hasChapterClicked.value = false
  // 重新加载数据
  fetchData()
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
    // 并行获取科目、错题和标签数据
    const [subjectsData, questionsData, tagsData] = await Promise.all([
      getSubjects(),
      getQuestions(),
      getErrorTags()
    ])
    
    subjects.value = subjectsData
    // 后端返回的数据包含 created_at 和 updated_at 等额外字段
    errors.value = questionsData as any[]
    
    // 提取所有唯一的标签名称
    const allTags = tagsData as any[]
    const uniqueTags = [...new Set(allTags.map(tag => tag.name))]
    availableTags.value = uniqueTags
    
    // 构建错题和标签的映射关系
    const tagMap = new Map<string, string[]>()
    allTags.forEach((tag: any) => {
      const questionId = tag.question_id
      const tagName = tag.name
      
      if (!tagMap.has(questionId)) {
        tagMap.set(questionId, [])
      }
      tagMap.get(questionId)!.push(tagName)
    })
    questionTagsMap.value = tagMap
  } catch (error) {
    console.error('获取数据失败:', error)
    errors.value = []
    subjects.value = []
    availableTags.value = []
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

// ============ 新增筛选功能 ============

// 已选筛选条件列表
const activeFilters = computed(() => {
  const filters_list = []
  
  // 科目
  if (filters.value.subject_id) {
    const subject = subjects.value.find(s => s.id === filters.value.subject_id)
    if (subject) {
      filters_list.push({ key: 'subject_id', label: subject.name })
    }
  }
  
  // 状态
  if (filters.value.status) {
    const statusMap: Record<string, string> = {
      pending: '待复习',
      reviewed: '已复习',
      mastered: '已掌握'
    }
    filters_list.push({ key: 'status', label: statusMap[filters.value.status] })
  }
  
  // 难度
  if (filters.value.difficulty) {
    const difficultyMap: Record<string, string> = {
      easy: '简单',
      medium: '中等',
      hard: '困难'
    }
    filters_list.push({ key: 'difficulty', label: difficultyMap[filters.value.difficulty] })
  }
  
  // 掌握程度
  if (filters.value.mastery) {
    const masteryMap: Record<string, string> = {
      unmastered: '未掌握',
      partial: '部分掌握',
      mastered: '已掌握'
    }
    filters_list.push({ key: 'mastery', label: masteryMap[filters.value.mastery] })
  }
  
  // 时间范围
  if (filters.value.date_range && filters.value.date_range !== 'all') {
    const dateRangeMap: Record<string, string> = {
      '7days': '最近7天',
      '30days': '最近30天',
      '90days': '最近90天'
    }
    filters_list.push({ key: 'date_range', label: dateRangeMap[filters.value.date_range] })
  }
  
  // 标签
  if (filters.value.tag) {
    filters_list.push({ key: 'tag', label: filters.value.tag })
  }
  
  // 书名
  if (filters.value.book) {
    filters_list.push({ key: 'book', label: filters.value.book })
  }
  
  // 章节
  if (filters.value.chapter) {
    filters_list.push({ key: 'chapter', label: filters.value.chapter })
  }
  
  // 知识点
  if (filters.value.knowledge) {
    filters_list.push({ key: 'knowledge', label: filters.value.knowledge })
  }
  
  return filters_list
})

// 移除单个筛选条件
const removeFilter = (key: string) => {
  switch (key) {
    case 'subject_id':
      filters.value.subject_id = ''
      filters.value.book = ''
      filters.value.chapter = ''
      filters.value.knowledge = ''
      break
    case 'status':
      filters.value.status = ''
      break
    case 'difficulty':
      filters.value.difficulty = ''
      break
    case 'mastery':
      filters.value.mastery = ''
      break
    case 'date_range':
      filters.value.date_range = 'all'
      break
    case 'tag':
      filters.value.tag = ''
      break
    case 'book':
      filters.value.book = ''
      filters.value.chapter = ''
      filters.value.knowledge = ''
      break
    case 'chapter':
      filters.value.chapter = ''
      filters.value.knowledge = ''
      break
    case 'knowledge':
      filters.value.knowledge = ''
      break
  }
}

// 清除所有筛选条件
const clearAllFilters = () => {
  filters.value.subject_id = ''
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  filters.value.status = ''
  filters.value.difficulty = ''
  filters.value.mastery = ''
  filters.value.date_range = 'all'
  filters.value.tag = ''
  filters.value.keyword = ''
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
        timestamp: question.updated_at || question.created_at || 0,
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
      // 难度筛选
      if (filters.value.difficulty) {
        if (filters.value.difficulty === 'easy' && error.difficulty > 1) return false
        if (filters.value.difficulty === 'medium' && (error.difficulty <= 1 || error.difficulty > 3)) return false
        if (filters.value.difficulty === 'hard' && error.difficulty <= 3) return false
      }
      // 掌握程度筛选
      if (filters.value.mastery) {
        // TODO: 从 SRS 数据获取 mastery，目前使用模拟数据
        const mastery = 50 // 默认值
        if (filters.value.mastery === 'unmastered' && mastery > 30) return false
        if (filters.value.mastery === 'partial' && (mastery <= 30 || mastery > 70)) return false
        if (filters.value.mastery === 'mastered' && mastery <= 70) return false
      }
      // 时间范围筛选
      if (filters.value.date_range && filters.value.date_range !== 'all') {
        const now = Date.now() / 1000 // 当前秒级时间戳
        const daysMap: Record<string, number> = {
          '7days': 7,
          '30days': 30,
          '90days': 90
        }
        const days = daysMap[filters.value.date_range] || 0
        const threshold = now - (days * 24 * 60 * 60)
        if (error.timestamp < threshold) return false
      }
      // 标签筛选
      if (filters.value.tag) {
        // 从映射中获取该错题的标签
        const questionTags = questionTagsMap.value.get(error.id) || []
        const hasTag = questionTags.includes(filters.value.tag)
        if (!hasTag) return false
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
    name: 'ManageDetail',
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

/* 搜索栏 - 第一行 */
.search-bar {
  margin-bottom: 12px;
}

.search-bar .search-box {
  width: 100%;
  margin: 0;
}

.search-bar .search-box input {
  width: 100%;
  height: 44px;
  font-size: 15px;
}

/* 筛选栏 - 第二行 */
.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  align-items: flex-start;
}

/* 已选筛选条件 */
.active-filters {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
  padding: 8px 12px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.active-filters-label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.filter-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: var(--primary-color);
  color: white;
  border-radius: 4px;
  font-size: 12px;
  white-space: nowrap;
}

.filter-tag-close {
  background: none;
  border: none;
  color: white;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
  margin-left: 2px;
  opacity: 0.8;
}

.filter-tag-close:hover {
  opacity: 1;
}

.clear-all-btn {
  padding: 4px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  margin-left: auto;
}

.clear-all-btn:hover {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
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

/* 关闭按钮 */
.cascade-close-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
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
  z-index: 10;
}

.cascade-close-btn:hover {
  background: #ff4d4f;
  color: white;
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

/* 下拉弹窗 */
.dropdown-popup {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  max-height: 300px;
  overflow-y: auto;
  animation: dropdownFadeIn 0.2s ease-out;
}

/* 标签下拉框特殊样式 */
.tag-dropdown {
  max-height: 250px;
}

@keyframes dropdownFadeIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 下拉项 */
.dropdown-item {
  padding: 10px 12px;
  font-size: 14px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dropdown-item:hover {
  background: var(--primary-color);
  color: white;
}

.dropdown-item.active {
  background: var(--primary-color);
  color: white;
  font-weight: 500;
}

.dropdown-item:active {
  transform: scale(0.98);
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

.cascade-column:first-child {
  padding-left: 8px;
}

.cascade-column:last-child {
  border-right: none;
  padding-right: 40px; /* 为关闭按钮留出空间 */
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