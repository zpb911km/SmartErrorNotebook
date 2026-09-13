<template>
  <div class="manage-page">
    <div class="page-heading">
      <h1>把错题整理成自己的知识库</h1>
      <p>找到薄弱环节，留住每一次思考。</p>
    </div>
    <div v-if="loadError" role="alert">
      {{ loadError }}
      <q-btn no-caps unelevated type="button" flat @click="fetchData">
        重新加载
      </q-btn>
    </div>
    <!-- 搜索栏 -->
    <div class="search-bar">
      <div class="search-box">
        <q-input
          ref="searchInput"
          v-model="filters.keyword"
          outlined
          dense
          aria-label="搜索题干、科目、书名、知识点...（模糊搜索）"
          type="text"
          placeholder="搜索题干、科目、书名、知识点...（模糊搜索）"
        />
      </div>
    </div>

    <LibraryFilters
      v-model="filters"
      v-model:difficulty-sort="difficultySort"
      v-model:mastery-sort="masterySort"
      :subjects="subjects"
      :sources="Array.from(sourceInfoMap.values())"
      :tags="availableTags"
    />
    <!-- 操作栏：导入/导出 -->
    <div class="action-bar">
      <span class="text-grey-7">{{ filteredErrors.length }} 道错题</span>
      <q-space />
      <q-btn
        flat
        color="primary"
        :label="selecting ? '退出选择' : '批量选择'"
        @click="toggleSelectionMode"
      />
      <q-btn
        v-if="selecting"
        flat
        label="全选当前结果"
        @click="selectedIds = filteredErrors.map((item) => item.id)"
      />
      <q-chip v-if="selecting" color="primary" text-color="white">
        已选 {{ selectedIds.length }} 题
      </q-chip>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="action-btn import-btn"
        @click="showImportModal = true"
      >
        <Icon name="plus" :size="16" />
        <span>导入</span>
      </q-btn>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="action-btn export-btn"
        :disable="selecting && !selectedIds.length"
        @click="showExportModal = true"
      >
        <Icon name="file-text" :size="16" />
        <span>导出</span>
      </q-btn>
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
        <q-btn
          no-caps
          unelevated
          type="button"
          flat
          class="filter-tag-close"
          @click="removeFilter(filter.key)"
        >
          <Icon name="x" :size="14" />
        </q-btn>
      </span>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="clear-all-btn"
        @click="clearAllFilters"
      >
        清除所有
      </q-btn>
    </div>

    <div class="error-list">
      <q-card
        v-for="error in filteredErrors"
        :key="error.id"
        flat
        bordered
        tabindex="0"
        role="button"
        class="error-card"
        @keydown.enter="activateQuestion(error)"
        @keydown.space.prevent="activateQuestion(error)"
        @click="activateQuestion(error)"
      >
        <q-checkbox
          v-if="selecting"
          v-model="selectedIds"
          :val="error.id"
          label="选择此题"
          @click.stop
          @keydown.stop
        />
        <!-- 上层：左边信息，右边标签 -->
        <div class="error-header">
          <div class="header-left">
            <span
              class="subject-tag"
              :style="getSubjectStyle(error.subjectId)"
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
          />
          <div class="error-footer">
            <span class="error-date">{{ error.date }}</span>
          </div>
        </div>
      </q-card>
    </div>

    <div v-if="isLoading" class="loading-state">
      <q-spinner color="primary" size="28px" />
      <div>加载中...</div>
    </div>

    <div
      v-if="!isLoading && !loadError && filteredErrors.length === 0"
      class="empty-illustration"
    >
      <div class="empty-icon" />
      <div class="empty-title">暂无错题</div>
      <div class="empty-desc">添加你的第一道错题，开始高效复习吧</div>
    </div>

    <!-- 导出弹窗 -->
    <ExportModal
      v-if="showExportModal"
      :questions="exportQuestions"
      @close="showExportModal = false"
    />

    <!-- 导入弹窗 -->
    <ImportModal
      v-if="showImportModal"
      :initial-data="pendingImportData"
      @close="handleImportModalClose"
      @import-complete="handleImportComplete"
    />
  </div>
</template>

<script setup lang="ts">
import type { QInput } from 'quasar'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ExportModal from '../components/ExportModal.vue'
import ImportModal from '../components/ImportModal.vue'
import LibraryFilters from '../components/LibraryFilters.vue'
import { useLatestRequest } from '../composables/useLatestRequest'
import { loadQuestionLibrary } from '../services/questionQueries'
import { clearPendingImport, importStore } from '../stores/importStore'
import type { Source, SrsData } from '../types'
import type { Subject } from '../types'
import type { QuestionView } from '../types/questionView'
import { renderMarkdown } from '../utils/markdown'
import { timestampSeconds } from '../utils/questionDisplay'

const router = useRouter()
const route = useRoute()
const searchInput = ref<QInput | null>(null)
const selecting = ref(false)
const selectedIds = ref<string[]>([])
function toggleSelectionMode() {
  selecting.value = !selecting.value
  selectedIds.value = []
}
function activateQuestion(question: { id: string }) {
  if (!selecting.value) {
    viewError(question)
    return
  }
  selectedIds.value = selectedIds.value.includes(question.id)
    ? selectedIds.value.filter((id) => id !== question.id)
    : [...selectedIds.value, question.id]
}
const exportQuestions = computed(() =>
  selecting.value
    ? filteredErrors.value.filter((item) => selectedIds.value.includes(item.id))
    : filteredErrors.value
)

// 本地筛选状态
const filters = ref({
  subjectId: '',
  book: '',
  chapter: '',
  knowledge: '',
  keyword: '',
  date_range: 'all', // '7days' | '30days' | '90days' | 'all'
  tags: [] as string[] // 标签名称数组（多选）
})

// Changing filters clears the batch selection.
watch(
  () => filters.value,
  () => {
    selectedIds.value = []
  },
  { deep: true }
)

// Current API fields plus explicitly loaded display relationships.
const errors = ref<QuestionView[]>([])
const subjects = ref<Subject[]>([])
const availableTags = ref<string[]>([])
const isLoading = ref(true)
const loadError = ref('')

// 错题和标签的映射关系（questionId -> 标签名称数组）
const questionTagsMap = ref<Map<string, string[]>>(new Map())

// 错题和来源的映射关系（sourceId -> 来源信息）
const sourceInfoMap = ref<Map<string | null, Source>>(new Map())

// SRS 数据缓存
const srsDataMap = ref<Map<string, SrsData>>(new Map())

// 排序状态
const difficultySort = ref<'asc' | 'desc' | 'none'>('none')
const masterySort = ref<'asc' | 'desc' | 'none'>('none')

// 导入/导出弹窗状态
const showExportModal = ref(false)
const showImportModal = ref(false)

/** 待导入数据直接从全局 importStore 读取 */
const pendingImportData = computed(() => importStore.pendingData)

// 导入完成后刷新数据
const handleImportComplete = () => {
  clearPendingImport()
  fetchData()
}

// 导入弹窗关闭时，清除待处理数据
const handleImportModalClose = () => {
  clearPendingImport()
  showImportModal.value = false
}

/** 当全局 store 有数据时，自动弹出导入弹窗 */
const checkPendingImport = () => {
  if (importStore.pendingData && !showImportModal.value) {
    showImportModal.value = true
  }
}

const focusSearch = () => {
  nextTick(() => searchInput.value?.focus())
}

// 从数据库获取数据
const beginLoad = useLatestRequest()
const fetchData = async () => {
  const isCurrent = beginLoad()
  isLoading.value = true
  loadError.value = ''
  try {
    const library = await loadQuestionLibrary()
    if (!isCurrent()) return
    subjects.value = library.subjects
    errors.value = library.items
    availableTags.value = [
      ...new Set(
        library.tags
          .filter((tag) => !tag.name.startsWith('[已删除]'))
          .map((tag) => tag.name)
      )
    ]
    questionTagsMap.value = new Map(
      library.items.map((question) => [
        question.id,
        question.tags
          .filter((tag) => !tag.name.startsWith('[已删除]'))
          .map((tag) => tag.name)
      ])
    )
    sourceInfoMap.value = new Map(
      library.sources.map((source) => [source.id, source])
    )
    srsDataMap.value = new Map(library.srs.map((srs) => [srs.questionId, srs]))
  } catch (error) {
    if (!isCurrent()) return
    loadError.value = '题目加载失败，请重试。'
    console.error('获取数据失败:', error)
    errors.value = []
    subjects.value = []
    availableTags.value = []
  } finally {
    if (isCurrent()) isLoading.value = false
  }
}

// 格式化日期（后端返回的是秒级时间戳）
const formatDate = (timestamp: number) => {
  if (!timestamp) return ''
  // 后端返回的是秒级时间戳，需要转换为毫秒
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN')
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
  if (filters.value.subjectId) {
    const subject = subjects.value.find((s) => s.id === filters.value.subjectId)
    if (subject) {
      filters_list.push({ key: 'subjectId', label: subject.name })
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
    case 'subjectId':
      filters.value.subjectId = ''
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
  difficultySort.value = 'none'
  masterySort.value = 'none'
  filters.value.subjectId = ''
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  filters.value.date_range = 'all'
  filters.value.tags = []
  filters.value.keyword = ''
}

onMounted(() => {
  // 从其他页面跳转过来时聚焦搜索框
  if (route.query.focus === 'search') {
    focusSearch()
  }
  // 监听自定义事件
  window.addEventListener('focus-library-search', focusSearch)
  // 加载数据
  fetchData()

  // 检查全局 store 中是否有待导入数据
  checkPendingImport()
})

// 监听路由 query 变化：App.vue 跳转过来时设置 import=1
watch(
  () => route.query.import,
  (val) => {
    if (val === '1') {
      checkPendingImport()
      // 清除 query 参数，刷新可重复触发
      router.replace({ query: { ...route.query, import: undefined } })
    }
  }
)

onUnmounted(() => {
  window.removeEventListener('focus-library-search', focusSearch)
})

// 过滤后的错题列表
const filteredErrors = computed(() => {
  const filtered = errors.value
    .map((question) => {
      const subject = subjects.value.find((s) => s.id === question.subject?.id)
      const difficulty = getDifficultyLevel(question.id)

      // 通过错题的 sourceId 获取来源信息
      // question.sourceId 是来源表的主键 ID
      const sourceId = question.sourceId
      const sourceInfo = sourceInfoMap.value.get(sourceId) || {
        book: '',
        chapter: '',
        knowledge: ''
      }

      return {
        id: question.id,
        subjectId: question.subject?.id ?? '',
        sourceId: question.sourceId,
        subjectName: subject?.name || '未知科目',
        difficulty,
        difficultyName: getDifficultyName(difficulty),
        content: question.stem || '',
        // 保留原始字段供导出使用
        stem: question.stem || '',
        correctAnswer: question.correctAnswer || '',
        explanation: question.explanation || '',
        // 后端返回的是秒级时间戳
        date: formatDate(
          timestampSeconds(question.updatedAt || question.createdAt)
        ),
        timestamp: timestampSeconds(question.updatedAt || question.createdAt),
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
      }

      // 科目筛选
      if (
        filters.value.subjectId &&
        error.subjectId !== filters.value.subjectId
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
          // 匹配错题内容
          const contentMatch = error.content.toLowerCase().includes(keyword)

          // 匹配科目名称
          const subjectMatch = error.subjectName.toLowerCase().includes(keyword)

          // 匹配书名
          const bookMatch = error.book?.toLowerCase().includes(keyword) || false

          // 匹配章节
          const chapterMatch =
            error.chapter?.toLowerCase().includes(keyword) || false

          // 匹配知识点
          const knowledgeMatch =
            error.knowledge?.toLowerCase().includes(keyword) || false

          // 匹配标签
          const tags = error.tags || []
          const tagMatch = tags.some((tag) =>
            tag.toLowerCase().includes(keyword)
          )

          // 只要任意一个维度匹配即可
          const isMatch =
            contentMatch ||
            subjectMatch ||
            bookMatch ||
            chapterMatch ||
            knowledgeMatch ||
            tagMatch

          if (!isMatch) {
            return false
          }
        }
      }
      return true
    })

  // 排序：难度筛选时按难度排序
  if (difficultySort.value !== 'none') {
    filtered.sort((a, b) => {
      const srsA = srsDataMap.value.get(a.id)
      const srsB = srsDataMap.value.get(b.id)
      const diffA = srsA?.difficulty || 5.0
      const diffB = srsB?.difficulty || 5.0
      return difficultySort.value === 'desc' ? diffB - diffA : diffA - diffB
    })
  }

  // 排序：掌握程度筛选时按掌握程度排序
  if (masterySort.value !== 'none') {
    // 计算掌握程度
    const calculateMastery = (questionId: string): number => {
      const srsData = srsDataMap.value.get(questionId)
      if (!srsData) return 50 // 默认值

      const reviewCount = srsData.reviewCount || 0
      const stability = srsData.stability || 0
      const recallRate = srsData.retrievability || 0

      // 综合计算掌握程度（0-100%）
      const reviewScore = Math.min(reviewCount / 10, 1) * 100
      const stabilityScore = Math.min(stability / 30, 1) * 100
      const recallScore = recallRate * 100

      return reviewScore * 0.3 + stabilityScore * 0.3 + recallScore * 0.4
    }

    filtered.sort((a, b) => {
      const masteryA = calculateMastery(a.id)
      const masteryB = calculateMastery(b.id)
      return masterySort.value === 'desc'
        ? masteryB - masteryA
        : masteryA - masteryB
    })
  }

  return filtered
})

/**
 * 获取滚动淡入延迟：前 10 张逐张 20ms，之后统一 200ms
 * 避免大量卡片时排队动画造成的卡顿感
 */

// 查看错题详情
const viewError = (error: { id: string }) => {
  router.push({
    name: 'ManageDetail',
    params: { id: error.id }
  })
}
</script>
