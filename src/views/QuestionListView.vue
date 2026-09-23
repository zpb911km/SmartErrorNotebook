<script setup lang="ts">
import type { QInput } from 'quasar'
import { computed, onMounted, ref, watch } from 'vue'

import ExportModal from '@/components/ExportModal.vue'
import ImportModal from '@/components/ImportModal.vue'
import LibraryFilterPanel, {
  createEmptyLibraryFilterPanelModelValue,
  type LibraryFilterPanelData,
  type LibraryFilterPanelModelValue
} from '@/components/LibraryFilterPanel.vue'
import { useQuestionLibrary } from '@/composables/useQuestionLibrary'
import { goQuestionDetail, goQuestionList } from '@/router'
import type { QuestionListIntent } from '@/router/types'
import { clearPendingImport, importStore } from '@/stores/importStore'
import type { SrsData } from '@/types'
import { renderMarkdown } from '@/utils/markdown'
import { timestampSeconds } from '@/utils/questionDisplay'

// ===== Component contract and local state =====
const props = defineProps<{ intent?: QuestionListIntent }>()

const keywordInputRef = ref<QInput | null>(null)
const filterPanelRef = ref<InstanceType<typeof LibraryFilterPanel> | null>(null)

// 获取科目样式
function getSubjectStyle(subjectId: string) {
  const subject = libraryData.subjects.find((s) => s.id === subjectId)
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

// 获取难度样式类
function getDifficultyClass(level: number) {
  if (level <= 1) return 'easy'
  if (level <= 2) return 'medium'
  return 'hard'
}

// ===== Library data and related records =====
const filterPanelModel = ref<LibraryFilterPanelModelValue>(
  createEmptyLibraryFilterPanelModelValue({
    filter: { keyword: '', dateRange: 'all', tagIds: [] },
    sort: { difficulty: 'none', mastery: 'none' }
  })
)

const { libraryData, refresh: fetchLibraryData } = useQuestionLibrary(
  () => filterPanelModel.value
)

// ===== Filter controls and derived display list =====
const filterPanelData = computed<LibraryFilterPanelData>(() => ({
  subjects: libraryData.subjects,
  sources: libraryData.sources,
  tags: libraryData.tags
}))

// 格式化日期（后端返回的是秒级时间戳）
const formatDate = (timestamp: number) => {
  if (!timestamp) return ''
  // 后端返回的是秒级时间戳，需要转换为毫秒
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN')
}

// 获取难度等级（基于 SRS 数据）
const getDifficultyLevel = (srsData: SrsData | null): number => {
  if (!srsData) {
    // 只在开发环境下输出警告，避免生产环境日志过多
    if (import.meta.env.DEV) {
      console.debug('题目暂无 SRS 数据，使用默认难度')
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

// TODO: 扩展IPC接口支持的filter参数，然后移除前端filter逻辑。
const filteredQuestions = computed(() => {
  const filtered = libraryData.questions
    .map((question) => {
      const difficulty = getDifficultyLevel(question.srs)

      return {
        id: question.id,
        subjectId: question.subject?.id ?? '',
        sourceId: question.sourceId,
        subjectName: question.subject?.name || '未知科目',
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
        // 来源信息
        book: question.source?.book ?? '',
        chapter: question.source?.chapter ?? '',
        knowledge: question.source?.knowledge ?? '',
        // 标签信息
        tags: question.tags.map((tag) => tag.name),
        srs: question.srs
      }
    })
    .filter((error) => {
      return (
        !filterPanelModel.value.filter.subjectId ||
        error.subjectId === filterPanelModel.value.filter.subjectId
      )
    })

  // 两种排序互斥：掌握程度由 IPC 排序，难度在本地排序。
  if (
    filterPanelModel.value.sort.difficulty !== 'none' &&
    filterPanelModel.value.sort.mastery === 'none'
  ) {
    filtered.sort((a, b) => {
      const diffA = a.srs?.difficulty || 5.0
      const diffB = b.srs?.difficulty || 5.0
      return filterPanelModel.value.sort.difficulty === 'desc'
        ? diffB - diffA
        : diffA - diffB
    })
  }

  return filtered
})

// ===== Batch selection and export scope =====
const isSelectionMode = ref(false)
const selectedQuestionIds = ref(new Set<string>())

function toggleSelectionMode() {
  isSelectionMode.value = !isSelectionMode.value
  selectedQuestionIds.value.clear()
}

function toggleQuestionSelection(id: string) {
  if (!selectedQuestionIds.value.delete(id)) {
    selectedQuestionIds.value.add(id)
  }
}

function onQuestionCardClick(id: string) {
  if (isSelectionMode.value) {
    toggleQuestionSelection(id)
  } else {
    goQuestionDetail(id)
  }
}

function selectAllQuestions() {
  selectedQuestionIds.value = new Set(
    filteredQuestions.value.map((item) => item.id)
  )
}

watch(
  () => filterPanelModel.value,
  () => selectedQuestionIds.value.clear(),
  { deep: true }
)

// ===== Import/export state and actions =====
type Modal = 'import' | 'export'

const activeModal = ref<Modal | null>(null)

function dismissModal(modal: Modal) {
  if (activeModal.value === modal) activeModal.value = null
}

/** 待导入数据直接从全局 importStore 读取 */
const pendingImportData = computed(() => importStore.pendingData)

function onImportModalDismiss() {
  if (activeModal.value !== 'import') return
  clearPendingImport()
  dismissModal('import')
}

function onImportModalComplete() {
  clearPendingImport()
  selectedQuestionIds.value.clear()
  void fetchLibraryData({ invalidateMetadata: true })
}

const questionsToExport = computed(() =>
  isSelectionMode.value
    ? filteredQuestions.value.filter((item) =>
        selectedQuestionIds.value.has(item.id)
      )
    : filteredQuestions.value
)

// ===== Intent lifecycle: route props in, canonical navigation out =====
async function handleIntent() {
  if (props.intent) {
    switch (props.intent) {
      case 'search':
        keywordInputRef.value?.focus()
        break

      case 'import':
        if (importStore.pendingData) {
          activeModal.value = 'import'
        }
        break
    }
    await goQuestionList({}, { replace: true })
  }
}

onMounted(handleIntent)

watch(() => props.intent, handleIntent)
</script>

<template>
  <div class="manage-page">
    <div class="page-heading">
      <h1>把错题整理成自己的知识库</h1>
      <p>找到薄弱环节，留住每一次思考。</p>
    </div>
    <div v-if="libraryData.status === 'error'" role="alert">
      题目加载失败，请重试。
      <q-btn no-caps unelevated type="button" flat @click="fetchLibraryData()">
        重新加载
      </q-btn>
    </div>
    <!-- 搜索栏 -->
    <div class="search-bar">
      <div class="search-box">
        <q-input
          ref="keywordInputRef"
          v-model="filterPanelModel.filter.keyword"
          outlined
          dense
          aria-label="搜索题干、解析或笔记"
          type="text"
          placeholder="搜索题干、解析或笔记"
        />
      </div>
    </div>

    <LibraryFilterPanel
      ref="filterPanelRef"
      v-model="filterPanelModel"
      :data="filterPanelData"
    />
    <!-- 操作栏：导入/导出 -->
    <div v-if="libraryData.status === 'ready'" class="action-bar">
      <span class="text-grey-7">{{ filteredQuestions.length }} 道错题</span>
      <q-space />
      <q-btn
        flat
        color="primary"
        :label="isSelectionMode ? '退出选择' : '批量选择'"
        @click="toggleSelectionMode"
      />
      <q-btn
        v-if="isSelectionMode"
        flat
        label="全选当前结果"
        @click="selectAllQuestions"
      />
      <q-chip v-if="isSelectionMode" color="primary" text-color="white">
        已选 {{ selectedQuestionIds.size }} 题
      </q-chip>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="action-btn import-btn"
        @click="activeModal = 'import'"
      >
        <AppIcon name="plus" :size="16" />
        <span>导入</span>
      </q-btn>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="action-btn export-btn"
        :disable="isSelectionMode && !selectedQuestionIds.size"
        @click="activeModal = 'export'"
      >
        <AppIcon name="file-text" :size="16" />
        <span>导出</span>
      </q-btn>
    </div>

    <div
      v-if="filterPanelRef?.activeFilterItems.length"
      class="active-filter-items"
    >
      <span class="active-filter-items-label">已选筛选：</span>
      <span
        v-for="filter in filterPanelRef.activeFilterItems"
        :key="filter.key"
        class="active-filter-item"
      >
        {{ filter.label }}
        <q-btn
          no-caps
          unelevated
          type="button"
          flat
          class="active-filter-item-close"
          :aria-label="`移除${filter.label}`"
          @click="filterPanelRef.clearFilterItem(filter.key)"
        >
          <AppIcon name="x" :size="14" />
        </q-btn>
      </span>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="clear-all-filter-items-btn"
        @click="filterPanelRef.clearAllFilterItems"
      >
        清除所有
      </q-btn>
    </div>

    <div v-if="libraryData.status === 'ready'" class="error-list">
      <q-card
        v-for="error in filteredQuestions"
        :key="error.id"
        flat
        bordered
        tabindex="0"
        role="button"
        class="error-card"
        @keydown.enter="onQuestionCardClick(error.id)"
        @keydown.space.prevent="onQuestionCardClick(error.id)"
        @click="onQuestionCardClick(error.id)"
      >
        <q-checkbox
          v-if="isSelectionMode"
          :model-value="selectedQuestionIds.has(error.id)"
          label="选择此题"
          @update:model-value="toggleQuestionSelection(error.id)"
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

    <div v-if="libraryData.status === 'loading'" class="loading-state">
      <q-spinner color="primary" size="28px" />
      <div>加载中...</div>
    </div>

    <div
      v-if="libraryData.status === 'ready' && filteredQuestions.length === 0"
      class="empty-illustration"
    >
      <div class="empty-icon" />
      <div class="empty-title">暂无错题</div>
      <div class="empty-desc">添加你的第一道错题，开始高效复习吧</div>
    </div>

    <!-- 导出弹窗 -->
    <ExportModal
      v-if="activeModal === 'export'"
      :questions="questionsToExport"
      @close="dismissModal('export')"
    />

    <!-- 导入弹窗 -->
    <ImportModal
      v-if="activeModal === 'import'"
      :initial-data="pendingImportData"
      @dismiss="onImportModalDismiss"
      @complete="onImportModalComplete"
    />
  </div>
</template>
