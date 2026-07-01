<template>
  <div class="manage-page">
    <!-- 搜索栏 -->
    <div class="search-bar">
      <div class="search-box" :class="{ blinking: isSearchBlinking }">
        <input
          type="text"
          v-model="filters.keyword"
          placeholder="搜索题干、科目、书名、知识点...（模糊搜索）"
          @focus="onSearchFocus"
        />
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <!-- 科目级联菜单 -->
      <div class="filter-select-wrapper" :class="{ 'dropdown-open': cascadeVisible }">
        <div class="custom-select" @click="toggleSubjectDropdown">
          <span class="custom-select-value">{{ selectedSubjectName || '选择科目' }}</span>
          <span class="custom-select-arrow" :class="{ rotated: cascadeVisible }">▼</span>
        </div>
        <!-- 级联弹窗 -->
        <div v-if="cascadeVisible" class="cascade-popup" @mouseenter="clearHideTimer" @mouseleave="hideAllMenus">
          <div class="cascade-column">
            <div class="column-title">科目</div>
            <div class="cascade-items">
              <div
                v-for="subj in subjects"
                :key="subj.id"
                class="cascade-item"
                :class="{ active: filters.subject_id === subj.id }"
                @click="handleSubjectClick(subj.id)"
              >
                {{ subj.name }}
                <span v-if="booksOf(subj.id).length" class="arrow-indicator">›</span>
              </div>
            </div>
          </div>
          <div v-if="currentSubjectId && books.length" class="cascade-column">
            <div class="column-title">书名</div>
            <div class="cascade-items">
              <div
                v-for="book in books"
                :key="book"
                class="cascade-item"
                :class="{ active: filters.book === book }"
                @click="handleBookClick(book)"
              >
                {{ book || '未分类' }}
                <span v-if="chaptersOf(book).length" class="arrow-indicator">›</span>
              </div>
            </div>
          </div>
          <div v-if="currentBook && chapters.length" class="cascade-column">
            <div class="column-title">章节</div>
            <div class="cascade-items">
              <div
                v-for="ch in chapters"
                :key="ch"
                class="cascade-item"
                :class="{ active: filters.chapter === ch }"
                @click="handleChapterClick(ch)"
              >
                {{ ch || '未分类' }}
                <span v-if="knowledges.length" class="arrow-indicator">›</span>
              </div>
            </div>
          </div>
          <div v-if="currentChapter && knowledges.length" class="cascade-column">
            <div class="column-title">知识点</div>
            <div class="cascade-items">
              <div
                v-for="kw in knowledges"
                :key="kw"
                class="cascade-item"
                :class="{ active: filters.knowledge === kw }"
                @click="selectKnowledge(kw)"
              >
                {{ kw || '未分类' }}
              </div>
            </div>
          </div>
          <button class="cascade-close-btn" @click="closeCascadeWindow">×</button>
        </div>
      </div>

      <!-- 标签下拉 -->
      <div class="filter-select-wrapper" :class="{ 'dropdown-open': tagDropdownVisible }">
        <div class="custom-select" @click="toggleTagDropdown">
          <span class="custom-select-value">{{ selectedTagsText || '选择标签' }}</span>
          <span class="custom-select-arrow" :class="{ rotated: tagDropdownVisible }">▼</span>
        </div>
        <div v-if="tagDropdownVisible" class="dropdown-popup tag-dropdown">
          <div
            v-for="tag in availableTags"
            :key="tag"
            class="dropdown-item"
            :class="{ active: filters.tags.includes(tag) }"
            @click="toggleTag(tag)"
          >
            <span class="checkbox">{{ filters.tags.includes(tag) ? '✓' : '' }}</span>
            {{ tag }}
          </div>
          <div v-if="filters.tags.length > 0" class="dropdown-item" style="border-top:1px solid var(--border-color);color:var(--primary-color);font-weight:500;" @click="clearTags">
            清除选择
          </div>
        </div>
      </div>

      <!-- 难度排序 -->
      <div class="filter-select-wrapper" :class="{ 'dropdown-open': difficultyDropdownVisible }">
        <div class="custom-select" @click="toggleDifficultyDropdown">
          <span class="custom-select-value">{{ difficultySort === 'none' ? '难度排序' : difficultySort === 'asc' ? '难度 ↑' : '难度 ↓' }}</span>
          <span class="custom-select-arrow" :class="{ rotated: difficultyDropdownVisible }">▼</span>
        </div>
        <div v-if="difficultyDropdownVisible" class="dropdown-popup">
          <div class="dropdown-item" :class="{ active: difficultySort === 'none' }" @click="setDifficultySort('none')">无排序</div>
          <div class="dropdown-item" :class="{ active: difficultySort === 'asc' }" @click="setDifficultySort('asc')">↑ 正序</div>
          <div class="dropdown-item" :class="{ active: difficultySort === 'desc' }" @click="setDifficultySort('desc')">↓ 倒序</div>
        </div>
      </div>

      <!-- 掌握程度排序 -->
      <div class="filter-select-wrapper" :class="{ 'dropdown-open': masteryDropdownVisible }">
        <div class="custom-select" @click="toggleMasteryDropdown">
          <span class="custom-select-value">{{ masterySort === 'none' ? '掌握程度' : masterySort === 'asc' ? '掌握 ↑' : '掌握 ↓' }}</span>
          <span class="custom-select-arrow" :class="{ rotated: masteryDropdownVisible }">▼</span>
        </div>
        <div v-if="masteryDropdownVisible" class="dropdown-popup">
          <div class="dropdown-item" :class="{ active: masterySort === 'none' }" @click="setMasterySort('none')">无排序</div>
          <div class="dropdown-item" :class="{ active: masterySort === 'asc' }" @click="setMasterySort('asc')">↑ 正序</div>
          <div class="dropdown-item" :class="{ active: masterySort === 'desc' }" @click="setMasterySort('desc')">↓ 倒序</div>
        </div>
      </div>

      <!-- 时间范围筛选 -->
      <div class="filter-select-wrapper" :class="{ 'dropdown-open': dateRangeDropdownVisible }">
        <div class="custom-select" @click="toggleDateRangeDropdown">
          <span class="custom-select-value">{{ dateRangeText || '时间范围' }}</span>
          <span class="custom-select-arrow" :class="{ rotated: dateRangeDropdownVisible }">▼</span>
        </div>
        <div v-if="dateRangeDropdownVisible" class="dropdown-popup">
          <div class="dropdown-item" :class="{ active: filters.date_range === 'all' }" @click="selectDateRange('all')">全部时间</div>
          <div class="dropdown-item" :class="{ active: filters.date_range === '7days' }" @click="selectDateRange('7days')">最近7天</div>
          <div class="dropdown-item" :class="{ active: filters.date_range === '30days' }" @click="selectDateRange('30days')">最近30天</div>
          <div class="dropdown-item" :class="{ active: filters.date_range === '90days' }" @click="selectDateRange('90days')">最近90天</div>
        </div>
      </div>
    </div>

    <!-- 操作栏：导入/导出 -->
    <div class="action-bar">
      <button class="action-btn import-btn" @click="showImportModal = true">
        <Icon name="plus" :size="16" />
        <span>导入</span>
      </button>
      <button class="action-btn export-btn" @click="showExportModal = true">
        <Icon name="file-text" :size="16" />
        <span>导出</span>
      </button>
    </div>

    <!-- 已选筛选条件 -->
    <div v-if="activeFilters.length > 0" class="active-filters">
      <span class="active-filters-label">已选筛选：</span>
      <span
        v-for="filter in activeFilters"
        :key="filter.key"
        class="filter-tag"
      >
        {{ filter.label }}
        <button @click="removeFilter(filter.key)" class="filter-tag-close">
          <Icon name="x" :size="14" />
        </button>
      </span>
      <button @click="clearAllFilters" class="clear-all-btn">清除所有</button>
    </div>

    <div class="error-list">
      <div
        v-for="(error, index) in filteredErrors"
        :key="error.id"
        v-scroll-reveal="{ delay: getRevealDelay(index), duration: 350 }"
        class="error-card"
        @click="viewError(error)"
      >
        <!-- 上层：左边信息，右边标签 -->
        <div class="error-header">
          <div class="header-left">
            <span
              class="subject-tag"
              :style="getSubjectStyle(error.subject_id)"
              >{{ error.subjectName }}</span
            >
            <span v-if="error.book" class="source-tag book-tag">{{
              error.book
            }}</span>
            <span v-if="error.chapter" class="source-tag chapter-tag">{{
              error.chapter
            }}</span>
            <span v-if="error.knowledge" class="source-tag knowledge-tag">{{
              error.knowledge
            }}</span>
            <span
              class="difficulty-tag"
              :class="getDifficultyClass(error.difficulty)"
              >{{ error.difficultyName }}</span
            >
          </div>
          <!-- 错因标签在右上角 -->
          <div
            v-if="error.tags && error.tags.length > 0"
            class="error-tags-inline"
          >
            <span
              v-for="tag in error.tags"
              :key="tag"
              class="tag-item-inline"
              >{{ tag }}</span
            >
          </div>
        </div>

        <!-- 下层：左边题干，右边时间靠下 -->
        <div class="error-body">
          <div
            class="error-content markdown-body"
            v-html="renderMarkdown(error.content)"
          ></div>
          <div class="error-footer">
            <span class="error-date">{{ error.date }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="filteredErrors.length === 0" class="empty-illustration">
      <div class="empty-icon"></div>
      <div class="empty-title">暂无错题</div>
      <div class="empty-desc">添加你的第一道错题，开始高效复习吧</div>
    </div>

    <!-- 导出弹窗 -->
    <ExportModal
      v-if="showExportModal"
      :questions="filteredErrors as any"
      @close="showExportModal = false"
    />

    <!-- 导入弹窗 -->
    <ImportModal
      v-if="showImportModal"
      @close="showImportModal = false"
      @import-complete="handleImportComplete"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getQuestions } from '../apis/errorQuestions'
import { getSubjects } from '../apis/subjects'
import {
  getBooks,
  getChapters,
  getKnowledges,
  getSources
} from '../apis/sources'
import { getFullErrorTags } from '../apis/errorTags'
import { getQuestionSRSStatus, createSRSData } from '../apis/srsData'
import ExportModal from '../components/ExportModal.vue'
import ImportModal from '../components/ImportModal.vue'
import type { Subject } from '../types'
import { marked } from 'marked'
import markedKatex from 'marked-katex-extension'

marked.use(
  markedKatex({
    throwOnError: false,
    output: 'html',
    nonStandard: true
  })
)

const renderer = new marked.Renderer()
marked.use({ renderer })

const router = useRouter()
const route = useRoute()

// 本地筛选状态
const filters = ref({
  subject_id: '',
  book: '',
  chapter: '',
  knowledge: '',
  keyword: '',
  date_range: 'all', // '7days' | '30days' | '90days' | 'all'
  tags: [] as string[] // 标签名称数组（多选）
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

// 错题和来源的映射关系（source_id -> 来源信息）
const sourceInfoMap = ref<Map<string, any>>(new Map())

// 筛选器引用

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

// 其他筛选器下拉框状态
const difficultyDropdownVisible = ref(false)
const masteryDropdownVisible = ref(false)
const dateRangeDropdownVisible = ref(false)
const tagDropdownVisible = ref(false)

// SRS 数据缓存
const srsDataMap = ref<Map<string, any>>(new Map())

// 排序状态
const difficultySort = ref<'asc' | 'desc' | 'none'>('none')
const masterySort = ref<'asc' | 'desc' | 'none'>('none')

// 导入/导出弹窗状态
const showExportModal = ref(false)
const showImportModal = ref(false)

// 导入完成后刷新数据
const handleImportComplete = () => {
  fetchData()
}

// 计算选中的科目名称
const selectedSubjectName = computed(() => {
  if (!filters.value.subject_id) return ''
  const subject = subjects.value.find((s) => s.id === filters.value.subject_id)
  return subject?.name || ''
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

/** 从已有数据中提取某科目所有的书 */
const booksOf = (subjectId: string): string[] => {
  const sourceIds = errors.value
    .filter((q: any) => q.subjectid === subjectId && q.sourceid)
    .map((q: any) => q.sourceid)
  const names = new Set<string>()
  for (const sid of sourceIds) {
    const info = sourceInfoMap.value.get(sid)
    if (info?.book) names.add(info.book)
  }
  return [...names]
}

/** 从已有数据中提取某书名下所有章节 */
const chaptersOf = (book: string): string[] => {
  const sourceIds = errors.value
    .filter((q: any) => {
      const info = sourceInfoMap.value.get(q.sourceid)
      return info?.book === book
    })
    .map((q: any) => q.sourceid)
  const names = new Set<string>()
  for (const sid of sourceIds) {
    const info = sourceInfoMap.value.get(sid)
    if (info?.chapter) names.add(info.chapter)
  }
  return [...names]
}

// 停止闪烁
// 切换科目下拉框
const toggleSubjectDropdown = () => {
  cascadeVisible.value = !cascadeVisible.value
  // 如果关闭下拉框，也隐藏级联菜单
  if (!cascadeVisible.value) {
    hideAllMenus()
  }
  // 关闭其他下拉框
  closeOtherDropdowns('subject')
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
  if (current !== 'subject') cascadeVisible.value = false
  if (current !== 'difficulty') difficultyDropdownVisible.value = false
  if (current !== 'mastery') masteryDropdownVisible.value = false
  if (current !== 'dateRange') dateRangeDropdownVisible.value = false
  if (current !== 'tag') tagDropdownVisible.value = false
}

// 设置难度排序
const setDifficultySort = (sort: 'asc' | 'desc' | 'none') => {
  difficultySort.value = sort
  difficultyDropdownVisible.value = false
}

// 设置掌握程度排序
const setMasterySort = (sort: 'asc' | 'desc' | 'none') => {
  masterySort.value = sort
  masteryDropdownVisible.value = false
}

// 选择时间范围
const selectDateRange = (dateRange: string) => {
  filters.value.date_range = dateRange
  dateRangeDropdownVisible.value = false
}

// 计算选中的标签文本
const selectedTagsText = computed(() => {
  if (filters.value.tags.length === 0) return ''
  if (filters.value.tags.length === 1) return filters.value.tags[0]
  return `${filters.value.tags[0]} +${filters.value.tags.length - 1}`
})

// 切换标签选择
const toggleTag = (tag: string) => {
  const index = filters.value.tags.indexOf(tag)
  if (index > -1) {
    filters.value.tags.splice(index, 1)
  } else {
    filters.value.tags.push(tag)
  }
}

// 清空所有标签选择
const clearTags = () => {
  filters.value.tags = []
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
  hasBookClicked.value = false
  hasChapterClicked.value = false
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

// 处理章节点击 - 设置点击状态并显示知识点
const handleChapterClick = async (chapter: string) => {
  console.log('=== 点击章节 ===')
  console.log('章节名称:', chapter)
  console.log('当前科目ID:', filters.value.subject_id)
  console.log('当前书名:', currentBook.value)

  hasChapterClicked.value = true

  // 更新当前章节
  currentChapter.value = chapter
  filters.value.chapter = chapter
  filters.value.knowledge = ''

  // 加载该章节的知识点
  if (filters.value.subject_id && currentBook.value && chapter) {
    try {
      console.log('开始获取知识点...')
      knowledges.value = await getKnowledges(
        currentBook.value,
        chapter,
        filters.value.subject_id
      )
      console.log('加载知识点成功:', knowledges.value)
      console.log('知识点数量:', knowledges.value.length)
    } catch (error) {
      console.error('获取知识点失败:', error)
      knowledges.value = []
    }
  } else {
    console.log('缺少必要参数，无法获取知识点')
    knowledges.value = []
  }

  // 重新加载数据以应用章节筛选
  console.log('重新加载数据...')
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

// 清除隐藏定时器（鼠标进入级联弹窗时调用）
const clearHideTimer = () => {
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
}

// 手动关闭级联窗口
const closeCascadeWindow = () => {
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
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

// 选择书名
const selectBook = (book: string) => {
  filters.value.book = book
  filters.value.chapter = ''
  filters.value.knowledge = ''
  // 重新加载数据以应用筛选
  fetchData()
}

// 选择知识点
const selectKnowledge = (knowledge: string) => {
  filters.value.knowledge = knowledge
  // 关闭所有菜单
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
    // 并行获取科目、错题、标签和来源数据
    const [subjectsData, questionsData, tagsData, sourcesData] =
      await Promise.all([
        getSubjects(),
        getQuestions(),
        getFullErrorTags(),
        getSources()
      ])

    subjects.value = subjectsData
    // 后端返回的数据包含 created_at 和 updated_at 等额外字段
    errors.value = questionsData as any[]

    // 批量获取 SRS 数据
    console.log('开始获取 SRS 数据...')
    const srsMap = new Map<string, any>()
    const questionsWithoutSRS: any[] = []

    const srsPromises = questionsData.map(async (question: any) => {
      try {
        const srsData = await getQuestionSRSStatus(question.id)
        if (srsData) {
          srsMap.set(question.id, srsData)
          console.log(`题目 ${question.id} 的 SRS 数据:`, {
            difficulty: srsData.difficulty,
            stability: srsData.stability,
            recall_rate: srsData.recall_rate,
            review_count: srsData.review_count
          })
        } else {
          console.warn(`题目 ${question.id} 没有 SRS 数据，将自动创建`)
          questionsWithoutSRS.push(question)
        }
      } catch (error) {
        console.warn(`获取题目 ${question.id} 的 SRS 数据失败:`, error)
        questionsWithoutSRS.push(question)
      }
    })

    await Promise.all(srsPromises)

    // 为没有 SRS 数据的题目创建 SRS 数据
    if (questionsWithoutSRS.length > 0) {
      console.log(`开始为 ${questionsWithoutSRS.length} 个题目创建 SRS 数据...`)
      const createPromises = questionsWithoutSRS.map(async (question: any) => {
        try {
          // 使用默认难度 5.0（中等）
          const srsData = await createSRSData(question.id, 5.0)
          srsMap.set(question.id, srsData)
          console.log(`为题目 ${question.id} 创建 SRS 数据成功:`, srsData)
        } catch (error) {
          console.error(`为题目 ${question.id} 创建 SRS 数据失败:`, error)
        }
      })

      await Promise.all(createPromises)
      console.log('SRS 数据创建完成')
    }

    srsDataMap.value = srsMap
    console.log('SRS 数据获取完成，总数:', srsMap.size)
    console.log('SRS 数据详情:', Array.from(srsMap.entries()))

    // 提取所有唯一的标签名称（过滤掉已删除的）
    const allTags = tagsData as any[]
    const activeTags = allTags.filter((tag) => !tag.name.startsWith('[已删除]'))
    const uniqueTags = [...new Set(activeTags.map((tag) => tag.name))]
    availableTags.value = uniqueTags

    // 构建错题和标签的映射关系（过滤掉已删除的）
    console.log(allTags)
    const tagMap = new Map<string, string[]>()
    activeTags.forEach((tag: any) => {
      const questionId = tag.question_id
      const tagName = tag.name

      if (!tagMap.has(questionId)) {
        tagMap.set(questionId, [])
      }
      tagMap.get(questionId)!.push(tagName)
    })
    questionTagsMap.value = tagMap
    console.log('tagMap:', tagMap)

    // 构建来源映射（source_id -> 来源信息）
    const sourceMap = new Map<string, any>()
    const allSources = sourcesData as any[]

    console.log('=== 来源数据调试 ===')
    console.log('所有来源数量:', allSources.length)
    console.log('所有来源:', allSources)

    // 按来源 ID 映射（不是 question_id）
    allSources.forEach((source: any) => {
      const sourceId = source.id // 来源表的主键 ID
      sourceMap.set(sourceId, {
        book: source.book || '',
        chapter: source.chapter || '',
        knowledge: source.knowledge || ''
      })
      console.log(`来源 ${sourceId} 的信息:`, {
        book: source.book,
        chapter: source.chapter,
        knowledge: source.knowledge
      })
    })

    console.log('来源映射数量:', sourceMap.size)
    console.log('====================')

    sourceInfoMap.value = sourceMap
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

// 渲染 Markdown
const renderMarkdown = (content: string) => {
  if (!content) return ''
  const normalized = content
    .replace(/\\\[/g, '$$$$')
    .replace(/\\\]/g, '$$$$')
    .replace(/\\\(/g, '$')
    .replace(/\\\)/g, '$')
  return marked.parse(normalized, { breaks: true, gfm: true }) as string
}

// 获取难度等级（基于 SRS 数据）
const getDifficultyLevel = (questionId: string): number => {
  const srsData = srsDataMap.value.get(questionId)
  if (!srsData) {
    // 只在开发环境下输出警告，避免生产环境日志过多
    if (import.meta.env.DEV) {
      console.debug(`题目 ${questionId} 暂无 SRS 数据，使用默认难度`)
    }
    return 2 // 默认中等
  }

  // SRS difficulty 范围是 [1.0, 10.0]
  // 映射到前端的 3 个等级：1=简单, 2=中等, 3=困难
  const difficulty = srsData.difficulty || 5.0
  if (difficulty <= 3.5) return 1 // 简单
  if (difficulty <= 6.5) return 2 // 中等
  return 3 // 困难
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
  const subject = subjects.value.find((s) => s.id === subjectId)
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
    const subject = subjects.value.find(
      (s) => s.id === filters.value.subject_id
    )
    if (subject) {
      filters_list.push({ key: 'subject_id', label: subject.name })
    }
  }

  // 时间范围
  if (filters.value.date_range && filters.value.date_range !== 'all') {
    const dateRangeMap: Record<string, string> = {
      '7days': '最近7天',
      '30days': '最近30天',
      '90days': '最近90天'
    }
    filters_list.push({
      key: 'date_range',
      label: dateRangeMap[filters.value.date_range]
    })
  }

  // 标签
  if (filters.value.tags.length > 0) {
    const label =
      filters.value.tags.length === 1
        ? filters.value.tags[0]
        : `${filters.value.tags[0]} +${filters.value.tags.length - 1}`
    filters_list.push({ key: 'tags', label })
  }

  // 难度排序
  if (difficultySort.value !== 'none') {
    const sortLabel =
      difficultySort.value === 'asc' ? '难度正序排序' : '难度倒序排序'
    filters_list.push({ key: 'difficulty_sort', label: sortLabel })
  }

  // 掌握程度排序
  if (masterySort.value !== 'none') {
    const sortLabel =
      masterySort.value === 'asc' ? '掌握程度正序排序' : '掌握程度倒序排序'
    filters_list.push({ key: 'mastery_sort', label: sortLabel })
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
    case 'date_range':
      filters.value.date_range = 'all'
      break
    case 'tags':
      filters.value.tags = []
      break
    case 'book':
      filters.value.book = ''
      break
    case 'chapter':
      filters.value.chapter = ''
      filters.value.knowledge = ''
      break
    case 'knowledge':
      filters.value.knowledge = ''
      break
    case 'difficulty_sort':
      difficultySort.value = 'none'
      break
    case 'mastery_sort':
      masterySort.value = 'none'
      break
  }
}

// 清除所有筛选条件
const clearAllFilters = () => {
  filters.value.subject_id = ''
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  filters.value.date_range = 'all'
  filters.value.tags = []
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
  let filtered = errors.value
    .map((question: any) => {
      const subject = subjects.value.find((s) => s.id === question.subjectid)
      const difficulty = getDifficultyLevel(question.id)
      const { status, statusText } = getReviewStatus(question.id)

      // 通过错题的 source_id 获取来源信息
      // question.sourceid 是来源表的主键 ID
      const sourceId = question.sourceid
      const sourceInfo = sourceInfoMap.value.get(sourceId) || {
        book: '',
        chapter: '',
        knowledge: ''
      }

      return {
        id: question.id,
        subject_id: question.subjectid,
        source_id: question.sourceid,
        subjectName: subject?.name || '未知科目',
        difficulty,
        difficultyName: getDifficultyName(difficulty),
        content: question.prompt || '',
        // 保留原始字段供导出使用
        prompt: question.prompt || '',
        answer: question.answer || '',
        analysis: question.analysis || '',
        // 后端返回的是秒级时间戳
        date: formatDate(question.updated_at || question.created_at || 0),
        timestamp: question.updated_at || question.created_at || 0,
        status,
        statusText,
        // 来源信息
        book: sourceInfo.book,
        chapter: sourceInfo.chapter,
        knowledge: sourceInfo.knowledge,
        // 标签信息
        tags: questionTagsMap.value.get(question.id) || []
      }
    })
    .filter((error) => {
      // 调试信息
      if (
        filters.value.book ||
        filters.value.chapter ||
        filters.value.knowledge
      ) {
        console.log(`错题 ${error.id} 筛选检查:`, {
          errorBook: error.book,
          errorChapter: error.chapter,
          errorKnowledge: error.knowledge,
          filterBook: filters.value.book,
          filterChapter: filters.value.chapter,
          filterKnowledge: filters.value.knowledge
        })
      }

      // 科目筛选
      if (
        filters.value.subject_id &&
        error.subject_id !== filters.value.subject_id
      ) {
        return false
      }
      // 书名筛选
      if (filters.value.book && error.book !== filters.value.book) {
        return false
      }
      // 章节筛选
      if (filters.value.chapter && error.chapter !== filters.value.chapter) {
        return false
      }
      // 知识点筛选
      if (
        filters.value.knowledge &&
        error.knowledge !== filters.value.knowledge
      ) {
        return false
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
        const threshold = now - days * 24 * 60 * 60
        if (error.timestamp < threshold) return false
      }
      // 标签筛选
      if (filters.value.tags.length > 0) {
        // 从映射中获取该错题的标签
        const questionTags = questionTagsMap.value.get(error.id) || []
        // 只要包含任意一个选中的标签即可
        const hasAnyTag = filters.value.tags.some((tag) =>
          questionTags.includes(tag)
        )
        if (!hasAnyTag) return false
      }
      // 关键词搜索 - 多维匹配
      if (filters.value.keyword) {
        const keyword = filters.value.keyword.toLowerCase().trim()
        if (keyword) {
          console.log('=== 搜索关键词 ===', keyword)

          // 匹配错题内容
          const contentMatch = error.content.toLowerCase().includes(keyword)
          console.log('内容匹配:', contentMatch)

          // 匹配科目名称
          const subjectMatch = error.subjectName.toLowerCase().includes(keyword)
          console.log('科目匹配:', subjectMatch, error.subjectName)

          // 匹配书名
          const bookMatch = error.book?.toLowerCase().includes(keyword) || false
          console.log('书名匹配:', bookMatch, error.book)

          // 匹配章节
          const chapterMatch =
            error.chapter?.toLowerCase().includes(keyword) || false
          console.log('章节匹配:', chapterMatch, error.chapter)

          // 匹配知识点
          const knowledgeMatch =
            error.knowledge?.toLowerCase().includes(keyword) || false
          console.log('知识点匹配:', knowledgeMatch, error.knowledge)

          // 匹配标签
          const tags = error.tags || []
          const tagMatch = tags.some((tag) =>
            tag.toLowerCase().includes(keyword)
          )
          console.log('标签匹配:', tagMatch, tags)

          // 只要任意一个维度匹配即可
          const isMatch =
            contentMatch ||
            subjectMatch ||
            bookMatch ||
            chapterMatch ||
            knowledgeMatch ||
            tagMatch
          console.log('最终匹配结果:', isMatch)

          if (!isMatch) {
            return false
          }
        }
      }
      return true
    })

  // 排序：难度筛选时按难度排序
  if (difficultySort.value !== 'none') {
    filtered.sort((a: any, b: any) => {
      const srsA = srsDataMap.value.get(a.id)
      const srsB = srsDataMap.value.get(b.id)
      const diffA = srsA?.difficulty || 5.0
      const diffB = srsB?.difficulty || 5.0
      return difficultySort.value === 'desc' ? diffB - diffA : diffA - diffB
    })

    // 输出排序结果日志
    if (filtered.length > 0) {
      const firstSRS = srsDataMap.value.get(filtered[0].id)
      const lastSRS = srsDataMap.value.get(filtered[filtered.length - 1].id)
      const firstDiff = firstSRS?.difficulty || 5.0
      const lastDiff = lastSRS?.difficulty || 5.0
      const sortType = difficultySort.value === 'asc' ? '正序' : '倒序'
      console.log(
        `难度排序完成 (${sortType}): ${firstDiff.toFixed(2)} ~ ${lastDiff.toFixed(2)}`
      )
    }
  }

  // 排序：掌握程度筛选时按掌握程度排序
  if (masterySort.value !== 'none') {
    // 计算掌握程度
    const calculateMastery = (questionId: string): number => {
      const srsData = srsDataMap.value.get(questionId)
      if (!srsData) return 50 // 默认值

      const reviewCount = srsData.review_count || 0
      const stability = srsData.stability || 0
      const recallRate = srsData.recall_rate || 0

      // 综合计算掌握程度（0-100%）
      const reviewScore = Math.min(reviewCount / 10, 1) * 100
      const stabilityScore = Math.min(stability / 30, 1) * 100
      const recallScore = recallRate * 100

      return reviewScore * 0.3 + stabilityScore * 0.3 + recallScore * 0.4
    }

    filtered.sort((a: any, b: any) => {
      const masteryA = calculateMastery(a.id)
      const masteryB = calculateMastery(b.id)
      return masterySort.value === 'desc'
        ? masteryB - masteryA
        : masteryA - masteryB
    })

    // 输出排序结果日志
    if (filtered.length > 0) {
      const firstMastery = calculateMastery(filtered[0].id)
      const lastMastery = calculateMastery(filtered[filtered.length - 1].id)
      const sortType = masterySort.value === 'asc' ? '正序' : '倒序'
      console.log(
        `掌握程度排序完成 (${sortType}): ${firstMastery.toFixed(1)}% ~ ${lastMastery.toFixed(1)}%`
      )
    }
  }

  return filtered
})

/**
 * 获取滚动淡入延迟：前 10 张逐张 20ms，之后统一 200ms
 * 避免大量卡片时排队动画造成的卡顿感
 */
const getRevealDelay = (index: number) => {
  if (index < 10) return index * 20
  return 200
}

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
  position: relative;
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

/* 操作栏 */
.action-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  justify-content: flex-end;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  border-color: var(--primary-color);
  background: var(--primary-light);
  color: var(--primary-color);
}

.import-btn:hover {
  border-color: var(--success-color);
  background: var(--success-light);
  color: var(--success-color);
}

.export-btn:hover {
  border-color: var(--primary-color);
  background: var(--primary-light);
  color: var(--primary-color);
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
  z-index: 1;
}

/* 当下拉框打开时，提升层级 */
.filter-select-wrapper.dropdown-open {
  z-index: 1002;
}

/* 向下展开的级联弹窗 — 宽度占满应用内容区，内部列使用 container query */
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
  z-index: 1001;
  box-sizing: border-box;
  width: calc(100vw - 40px);
  max-width: calc(100vw - 40px);
  padding-right: 40px;
  max-height: 500px;
  animation: cascadeSlideDown 0.2s ease-out;
  overflow-x: auto;
  container-type: inline-size;
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
  top: calc(100% + 4px);
  left: 0;
  right: auto;
  min-width: 150px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  z-index: 1001;
  max-height: 300px;
  overflow-y: auto;
  animation: dropdownFadeIn 0.2s ease-out;
}

/* 标签下拉框特殊样式 */
.tag-dropdown {
  max-height: 250px;
}

/* 复选框样式 */
.checkbox {
  display: inline-block;
  width: 16px;
  margin-right: 8px;
  color: var(--primary-color);
  font-weight: bold;
  text-align: center;
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

.sort-indicator {
  margin-left: 4px;
  font-size: 12px;
}

/* 级联列 — 列宽随容器平滑变化 80~220px */
.cascade-column {
  flex: 0 0 clamp(80px, 20cqi, 220px);
  width: clamp(80px, 20cqi, 220px);
  border-right: 1px solid var(--border-color);
  padding: 8px 0;
  max-height: 500px;
  overflow-y: auto;
  overflow-x: hidden;
}

.cascade-column:first-child {
  padding-left: 8px;
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
  0%,
  100% {
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
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  height: 200px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.error-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08), 0 2px 8px rgba(0, 0, 0, 0.06);
}

.error-card:active {
  transform: scale(0.98);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.error-header {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  flex: 1;
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

.source-tag {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.book-tag {
  background: #f3e5f5;
  color: #7b1fa2;
}

.chapter-tag {
  background: #e8f5e9;
  color: #43a047;
}

.knowledge-tag {
  background: #e0f2f1;
  color: #00796b;
}

.error-tags-inline {
  display: flex;
  gap: 4px;
  margin-left: auto;
  flex-wrap: wrap;
}

.tag-item-inline {
  padding: 3px 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 下层布局 */
.error-body {
  display: flex;
  flex: 1;
  gap: 12px;
  min-height: 0;
}

.error-content {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.5;
  flex: 1;
  overflow: hidden;
  position: relative;
}

.error-footer {
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  font-size: 12px;
  padding-bottom: 0;
  flex-shrink: 0;
}

.error-date {
  color: var(--text-secondary);
  white-space: nowrap;
}

/* Markdown 渲染内容溢出处理 */
.error-content.markdown-body {
  overflow: hidden;
  max-height: 100%;
}

/* Markdown 渲染样式 */
.error-content.markdown-body :deep(h1),
.error-content.markdown-body :deep(h2),
.error-content.markdown-body :deep(h3),
.error-content.markdown-body :deep(h4),
.error-content.markdown-body :deep(h5),
.error-content.markdown-body :deep(h6) {
  margin: 0.8em 0 0.4em;
  font-weight: 600;
  font-size: 1em;
}

.error-content.markdown-body :deep(p) {
  margin: 0.5em 0;
}

.error-content.markdown-body :deep(code) {
  background: rgba(25, 118, 210, 0.12);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.9em;
}

.error-content.markdown-body :deep(pre) {
  background: var(--code-bg, #0f172a);
  padding: 10px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.error-content.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  color: var(--code-text, #e2e8f0);
}

.error-content.markdown-body :deep(pre code.hljs) {
  background: transparent;
  color: var(--code-text, #e2e8f0);
}

.error-content.markdown-body :deep(ul),
.error-content.markdown-body :deep(ol) {
  padding-left: 20px;
  margin: 0.5em 0;
}

.error-content.markdown-body :deep(blockquote) {
  margin: 0.5em 0;
  padding-left: 10px;
  border-left: 3px solid var(--border-color);
  color: var(--text-secondary);
}

.error-content.markdown-body :deep(a) {
  color: var(--primary-color);
  text-decoration: underline;
}

.error-content.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}

.error-content.markdown-body :deep(th),
.error-content.markdown-body :deep(td) {
  border: 1px solid var(--border-color);
  padding: 6px 8px;
}

/* 旧的 footer 样式保留但不再使用 */
.error-footer-old {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  font-size: 12px;
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

<!-- 全局覆盖 highlight.js 在浅色模式下的颜色 -->
<style>
.error-list pre code.hljs {
  background: transparent !important;
  color: var(--code-text, #e2e8f0) !important;
}
.error-list pre {
  background: var(--code-bg, #0f172a) !important;
}
</style>
