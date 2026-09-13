<template>
  <div class="preview-page">
    <div v-if="loadError" role="alert">
      {{ loadError }}
      <q-btn no-caps unelevated type="button" flat @click="fetchData">
        重新加载
      </q-btn>
    </div>
    <div class="page-heading">
      <h1>给记忆一点温习的时间</h1>
      <p>根据掌握情况安排复习，也可以选择任意错题自由回顾。</p>
    </div>
    <LibraryFilters
      v-model="filters"
      :subjects="subjects"
      :sources="Array.from(sourceInfoMap.values())"
    />
    <!-- 已选筛选条件 -->
    <div v-if="activeFilters.length > 0" class="active-filters">
      <span class="active-filters-label">已选：</span>
      <span v-for="f in activeFilters" :key="f.key" class="filter-tag">
        {{ f.label }}
        <q-btn
          no-caps
          unelevated
          type="button"
          flat
          class="filter-tag-close"
          @click="removeFilter(f.key)"
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
        清除
      </q-btn>
    </div>

    <!-- 待复习列表 -->
    <div class="section-header">
      <h3>待复习</h3>
      <span class="section-count">{{ dueList.length }} 题</span>
    </div>

    <div v-if="dueList.length > 0" class="card-list">
      <q-card
        v-for="item in dueList"
        :key="item.id"
        flat
        bordered
        tabindex="0"
        role="button"
        class="error-card"
        @keydown.enter="reviewCard(item)"
        @keydown.space.prevent="reviewCard(item)"
        @click="reviewCard(item)"
      >
        <div class="error-header">
          <span class="subject-tag" :style="getSubjectStyle(item.subjectId)">{{
            item.subjectName
          }}</span>
          <span v-if="item.knowledge" class="source-tag knowledge-tag">{{
            item.knowledge
          }}</span>
          <span class="urgency-badge urgent">{{ item.urgencyLabel }}</span>
        </div>
        <div
          class="error-content markdown-body"
          v-html="renderMarkdown(item.prompt)"
        />
        <div class="error-footer">
          <span class="meta-item">⏱ {{ item.lastReviewLabel }}</span>
          <span class="meta-item"
            ><Icon name="target" :size="16" /> 预期回忆
            {{ item.recallPercent }}%</span
          >
        </div>
      </q-card>
    </div>

    <!-- 分割线 -->
    <div class="section-divider">
      <span>无需复习 — {{ notDueList.length }} 题</span>
    </div>

    <!-- 无需复习列表 -->
    <div v-if="notDueList.length > 0" class="card-list">
      <q-card
        v-for="item in notDueList"
        :key="item.id"
        flat
        bordered
        tabindex="0"
        role="button"
        class="error-card"
        @keydown.enter="reviewCard(item)"
        @keydown.space.prevent="reviewCard(item)"
        @click="reviewCard(item)"
      >
        <div class="error-header">
          <span class="subject-tag" :style="getSubjectStyle(item.subjectId)">{{
            item.subjectName
          }}</span>
          <span v-if="item.knowledge" class="source-tag knowledge-tag">{{
            item.knowledge
          }}</span>
          <span class="urgency-badge upcoming">{{ item.urgencyLabel }}</span>
        </div>
        <div
          class="error-content markdown-body"
          v-html="renderMarkdown(item.prompt)"
        />
        <div class="error-footer">
          <span class="meta-item">📅 {{ item.nextReviewLabel }}</span>
          <span class="meta-item"
            ><Icon name="chart-column" :size="16" /> 记忆强度
            {{ item.stabilityText }}</span
          >
        </div>
      </q-card>
    </div>

    <!-- 空状态 -->
    <div v-if="isLoading" class="loading-state">
      <q-spinner color="primary" size="28px" />
      <div>加载中...</div>
    </div>

    <div
      v-if="!isLoading && !loadError && allFiltered.length === 0"
      class="empty-illustration"
    >
      <div class="empty-icon" />
      <div class="empty-title">没有符合条件的错题</div>
      <div class="empty-desc">调整筛选条件，或添加更多错题吧</div>
    </div>

    <!-- FAB -->
    <q-btn
      v-if="dueList.length > 0"
      no-caps
      unelevated
      type="button"
      color="primary"
      class="fab"
      @click="startReview"
    >
      <span class="fab-icon">▶</span>
      <span class="fab-text">开始复习</span>
      <span class="fab-badge">{{ dueList.length }}</span>
    </q-btn>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import LibraryFilters from '../components/LibraryFilters.vue'
import { useLatestRequest } from '../composables/useLatestRequest'
import { loadQuestionLibrary } from '../services/questionQueries'
import type { ReviewCard } from '../services/reviewStore'
import { setReviewQueue } from '../services/reviewStore'
import type { Source, SrsData } from '../types'
import type { Subject } from '../types'
import type { QuestionView } from '../types/questionView'
import { renderMarkdown } from '../utils/markdown'
import { timestampSeconds } from '../utils/questionDisplay'

const router = useRouter()

// ============ Data ============
const subjects = ref<Subject[]>([])
const questions = ref<QuestionView[]>([])
const srsCards = ref<SrsData[]>([])
const questionTagsMap = ref<Map<string, string[]>>(new Map())
const sourceInfoMap = ref<Map<string | null, Source>>(new Map())
const isLoading = ref(true)
const loadError = ref('')

const filters = ref({
  subjectId: '',
  book: '',
  chapter: '',
  knowledge: ''
})

const activeFilters = computed(() => {
  const list: { key: string; label: string }[] = []
  if (filters.value.subjectId) {
    const s = subjects.value.find((x) => x.id === filters.value.subjectId)
    if (s) list.push({ key: 'subjectId', label: s.name })
  }
  if (filters.value.book)
    list.push({ key: 'book', label: `📖 ${filters.value.book}` })
  if (filters.value.chapter)
    list.push({ key: 'chapter', label: `📑 ${filters.value.chapter}` })
  if (filters.value.knowledge)
    list.push({ key: 'knowledge', label: `🏷 ${filters.value.knowledge}` })
  return list
})

const now = () => Math.floor(Date.now() / 1000)

// Build merged items
interface MergedItem {
  id: string
  questionId: string
  subjectId: string
  subjectName: string
  prompt: string
  book: string
  chapter: string
  knowledge: string
  srs: SrsData
  stability: number
  difficulty: number
  recallRate: number
  nextReviewAt: number | null
  lastReviewAt: number | null
  reviewCount: number
  // Computed display fields
  recallPercent: number
  urgencyLabel: string
  lastReviewLabel: string
  nextReviewLabel: string
  stabilityText: string
  isDue: boolean
}

const mergedItems = computed(() => {
  const srsByQId = new Map<string, SrsData>()
  for (const srs of srsCards.value) {
    srsByQId.set(srs.questionId, srs)
  }

  const items: MergedItem[] = []
  for (const q of questions.value) {
    const srs = srsByQId.get(q.id)
    if (!srs) continue

    const subject = subjects.value.find((s) => s.id === q.subject?.id)
    const sourceId = q.sourceId
    const sourceInfo = sourceInfoMap.value.get(sourceId)

    const recallRate = srs.retrievability ?? 0
    const recallPercent = Math.round(recallRate * 100)
    const n = now()
    const lastAt = srs.lastReviewAt ? timestampSeconds(srs.lastReviewAt) : null
    const daysSinceLast = lastAt
      ? Math.max(0, Math.floor((n - lastAt) / 86400))
      : -1
    const nextAt = srs.nextReviewAt ? timestampSeconds(srs.nextReviewAt) : null
    const daysUntilNext = nextAt ? Math.floor((nextAt - n) / 86400) : null
    const isDue = srs.isDue

    console.log(srs, isDue, n)

    let urgencyLabel: string
    let lastReviewLabel: string
    let nextReviewLabel: string
    let stabilityText: string
    const stab = srs.stability ?? 0

    if (isDue) {
      if (recallPercent < 30) urgencyLabel = '🔴 复习'
      else if (recallPercent < 60) urgencyLabel = '🟡 复习'
      else urgencyLabel = '🟢 复习'
      lastReviewLabel = daysSinceLast >= 0 ? `${daysSinceLast} 天前` : '未复习'
      nextReviewLabel = ''
      stabilityText = `${stab.toFixed(1)} 天`
    } else {
      if (daysUntilNext !== null) {
        if (daysUntilNext <= 0) urgencyLabel = '🔴 今天'
        else if (daysUntilNext === 1) urgencyLabel = '🟡 明天'
        else if (daysUntilNext <= 3) urgencyLabel = `🟡 ${daysUntilNext} 天后`
        else if (daysUntilNext <= 7) urgencyLabel = `🟢 ${daysUntilNext} 天后`
        else urgencyLabel = `🟢 ${daysUntilNext} 天后`
      } else {
        urgencyLabel = '⚪ 待安排'
      }
      nextReviewLabel =
        daysUntilNext !== null ? `${daysUntilNext} 天后` : '待安排'
      lastReviewLabel = daysSinceLast >= 0 ? `${daysSinceLast} 天前` : '未复习'
      stabilityText = `${stab.toFixed(1)} 天`
    }

    items.push({
      id: q.id,
      questionId: q.id,
      subjectId: q.subject?.id ?? '',
      subjectName: subject?.name || '未知',
      prompt: q.stem || '',
      book: sourceInfo?.book || '',
      chapter: sourceInfo?.chapter || '',
      knowledge: sourceInfo?.knowledge || '',
      srs,
      stability: stab,
      difficulty: srs.difficulty ?? 5,
      recallRate,
      nextReviewAt: nextAt,
      lastReviewAt: lastAt,
      reviewCount: srs.reviewCount ?? 0,
      recallPercent,
      urgencyLabel,
      lastReviewLabel,
      nextReviewLabel,
      stabilityText,
      isDue: isDue
    })
  }
  return items
})

const filteredItems = computed(() => {
  return mergedItems.value.filter((item) => {
    if (filters.value.subjectId && item.subjectId !== filters.value.subjectId)
      return false
    if (filters.value.book && item.book !== filters.value.book) return false
    if (filters.value.chapter && item.chapter !== filters.value.chapter)
      return false
    if (filters.value.knowledge && item.knowledge !== filters.value.knowledge)
      return false
    return true
  })
})

const dueList = computed(() => {
  return filteredItems.value
    .filter((item) => {
      return item.isDue
    })
    .sort((a, b) => a.recallRate - b.recallRate)
})

const notDueList = computed(() => {
  return filteredItems.value
    .filter((item) => {
      return !item.isDue
    })
    .sort((a, b) => (a.nextReviewAt ?? Infinity) - (b.nextReviewAt ?? Infinity))
})

const allFiltered = computed(() => [...dueList.value, ...notDueList.value])

// ============ Methods ============
function getSubjectStyle(subjectId: string) {
  const subject = subjects.value.find((s) => s.id === subjectId)
  if (subject?.color) {
    return { backgroundColor: `${subject.color}20`, color: subject.color }
  }
  return { backgroundColor: '#e3f2fd', color: '#1976d2' }
}

function removeFilter(key: string) {
  if (key === 'subjectId') {
    filters.value.subjectId = ''
    filters.value.book = ''
    filters.value.chapter = ''
    filters.value.knowledge = ''
  } else if (key === 'book') {
    filters.value.book = ''
    filters.value.chapter = ''
    filters.value.knowledge = ''
  } else if (key === 'chapter') {
    filters.value.chapter = ''
    filters.value.knowledge = ''
  } else if (key === 'knowledge') {
    filters.value.knowledge = ''
  }
}

function clearAllFilters() {
  filters.value.subjectId = ''
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
}

function buildReviewCard(item: MergedItem): ReviewCard {
  return {
    questionId: item.questionId,
    srs: item.srs,
    question: questions.value.find((q) => q.id === item.questionId)!,
    subjectName: item.subjectName
  }
}

function reviewCard(item: MergedItem) {
  setReviewQueue([buildReviewCard(item)], 'all')
  router.push({ name: 'ReviewDetail' })
}

function startReview() {
  const queue = dueList.value.map(buildReviewCard)
  if (queue.length === 0) return
  setReviewQueue(queue, 'due')
  router.push({ name: 'ReviewDetail' })
}

// ============ Lifecycle ============
const beginLoad = useLatestRequest()
const fetchData = async () => {
  const isCurrent = beginLoad()
  isLoading.value = true
  loadError.value = ''
  try {
    const library = await loadQuestionLibrary()
    if (!isCurrent()) return
    subjects.value = library.subjects
    questions.value = library.items
    srsCards.value = library.srs
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
  } catch {
    if (isCurrent()) loadError.value = '复习数据加载失败，请重试。'
  } finally {
    if (isCurrent()) isLoading.value = false
  }
}
onMounted(fetchData)
</script>
