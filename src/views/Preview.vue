<template>
  <div class="preview-page">
    <!-- 筛选栏 -->
    <div class="filter-bar">
      <div class="filter-select-wrapper">
        <div class="custom-select" @click="toggleSubjectDropdown">
          <div class="custom-select-value">{{ selectedSubjectName || '全部科目' }}</div>
          <span class="custom-select-arrow" :class="{ rotated: subjectDropdownVisible }">▼</span>
        </div>

        <div v-if="subjectDropdownVisible || cascadeVisible" class="cascade-popup">
          <button class="cascade-close-btn" @click="closeCascadeWindow">×</button>

          <div class="cascade-column">
            <div class="column-title">科目</div>
            <div class="column-items">
              <div class="cascade-item" :class="{ active: !filters.subject_id }" @click="selectSubject('')">
                全部
              </div>
              <div v-for="subj in subjects" :key="subj.id" class="cascade-item"
                :class="{ active: filters.subject_id === subj.id }" @click="handleSubjectClick(subj.id)">
                {{ subj.name }}
                <span class="arrow-indicator">›</span>
              </div>
            </div>
          </div>

          <div v-if="currentSubjectId && books.length > 0" class="cascade-column">
            <div class="column-title">书名</div>
            <div class="column-items">
              <div class="cascade-item" :class="{ active: !filters.book }" @click="handleBookClick('')">
                全部
              </div>
              <div v-for="b in books" :key="b" class="cascade-item"
                :class="{ active: filters.book === b }" @click="handleBookClick(b)">
                {{ b || '未分类' }}
                <span class="arrow-indicator">›</span>
              </div>
            </div>
          </div>

          <div v-if="currentBook && chapters.length > 0" class="cascade-column">
            <div class="column-title">章节</div>
            <div class="column-items">
              <div class="cascade-item" :class="{ active: !filters.chapter }" @click="handleChapterClick('')">
                全部
              </div>
              <div v-for="ch in chapters" :key="ch" class="cascade-item"
                :class="{ active: filters.chapter === ch }" @click="handleChapterClick(ch)">
                {{ ch || '未分类' }}
                <span class="arrow-indicator">›</span>
              </div>
            </div>
          </div>

          <div v-if="currentChapter && knowledges.length > 0" class="cascade-column">
            <div class="column-title">知识点</div>
            <div class="column-items">
              <div class="cascade-item" :class="{ active: !filters.knowledge }" @click="selectKnowledge('')">
                全部
              </div>
              <div v-for="k in knowledges" :key="k" class="cascade-item"
                :class="{ active: filters.knowledge === k }" @click="selectKnowledge(k)">
                {{ k || '未分类' }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 已选筛选条件 -->
    <div v-if="activeFilters.length > 0" class="active-filters">
      <span class="active-filters-label">已选：</span>
      <span v-for="f in activeFilters" :key="f.key" class="filter-tag">
        {{ f.label }}
        <button @click="removeFilter(f.key)" class="filter-tag-close">×</button>
      </span>
      <button @click="clearAllFilters" class="clear-all-btn">清除</button>
    </div>

    <!-- 待复习列表 -->
    <div class="section-header">
      <h3>待复习</h3>
      <span class="section-count">{{ dueList.length }} 题</span>
    </div>

    <div v-if="dueList.length > 0" class="card-list">
      <div v-for="item in dueList" :key="item.id" class="error-card" @click="reviewCard(item)">
        <div class="error-header">
          <span class="subject-tag" :style="getSubjectStyle(item.subjectId)">{{ item.subjectName }}</span>
          <span v-if="item.knowledge" class="source-tag knowledge-tag">{{ item.knowledge }}</span>
          <span class="urgency-badge urgent">{{ item.urgencyLabel }}</span>
        </div>
        <div class="error-content markdown-body" v-html="renderMarkdown(item.prompt)"></div>
        <div class="error-footer">
          <span class="meta-item">⏱ {{ item.lastReviewLabel }}</span>
          <span class="meta-item">🎯 掌握率 {{ item.recallPercent }}%</span>
        </div>
      </div>
    </div>

    <!-- 分割线 -->
    <div class="section-divider">
      <span>无需复习 — {{ notDueList.length }} 题</span>
    </div>

    <!-- 无需复习列表 -->
    <div v-if="notDueList.length > 0" class="card-list">
      <div v-for="item in notDueList" :key="item.id" class="error-card" @click="reviewCard(item)">
        <div class="error-header">
          <span class="subject-tag" :style="getSubjectStyle(item.subjectId)">{{ item.subjectName }}</span>
          <span v-if="item.knowledge" class="source-tag knowledge-tag">{{ item.knowledge }}</span>
          <span class="urgency-badge upcoming">{{ item.urgencyLabel }}</span>
        </div>
        <div class="error-content markdown-body" v-html="renderMarkdown(item.prompt)"></div>
        <div class="error-footer">
          <span class="meta-item">📅 {{ item.nextReviewLabel }}</span>
          <span class="meta-item">📊 稳定性 {{ item.stabilityText }}</span>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="allFiltered.length === 0" class="empty-state">
      <div class="empty-icon">📭</div>
      <p>没有符合条件的错题</p>
    </div>

    <!-- FAB -->
    <button v-if="dueList.length > 0" class="fab" @click="startReview">
      <span class="fab-icon">▶</span>
      <span class="fab-text">开始复习</span>
      <span class="fab-badge">{{ dueList.length }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSubjects } from '../apis/subjects'
import { getQuestions } from '../apis/errorQuestions'
import { getAllSRSStatus } from '../apis/srs'
import { getBooks, getChapters, getKnowledges, getSources } from '../apis/sources'
import { getFullErrorTags } from '../apis/errorTags'
import { setReviewQueue } from '../services/reviewStore'
import type { Subject } from '../types'
import { marked } from 'marked'

const router = useRouter()

// ============ Data ============
const subjects = ref<Subject[]>([])
const questions = ref<any[]>([])
const srsCards = ref<any[]>([])
const questionTagsMap = ref<Map<string, string[]>>(new Map())
const sourceInfoMap = ref<Map<string, any>>(new Map())

const filters = ref({
  subject_id: '',
  book: '',
  chapter: '',
  knowledge: '',
})

// Cascade state
const subjectDropdownVisible = ref(false)
const cascadeVisible = ref(false)
const currentSubjectId = ref<string | null>(null)
const currentBook = ref<string | null>(null)
const currentChapter = ref<string | null>(null)
const books = ref<string[]>([])
const chapters = ref<string[]>([])
const knowledges = ref<string[]>([])

// ============ Computed ============
const selectedSubjectName = computed(() => {
  if (!filters.value.subject_id) return ''
  const subj = subjects.value.find(s => s.id === filters.value.subject_id)
  return subj?.name || ''
})

const activeFilters = computed(() => {
  const list: { key: string; label: string }[] = []
  if (filters.value.subject_id) {
    const s = subjects.value.find(x => x.id === filters.value.subject_id)
    if (s) list.push({ key: 'subject_id', label: s.name })
  }
  if (filters.value.book) list.push({ key: 'book', label: `📖 ${filters.value.book}` })
  if (filters.value.chapter) list.push({ key: 'chapter', label: `📑 ${filters.value.chapter}` })
  if (filters.value.knowledge) list.push({ key: 'knowledge', label: `🏷 ${filters.value.knowledge}` })
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
  knowledge: string
  srs: any
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
  const srsByQId = new Map<string, any>()
  for (const srs of srsCards.value) {
    srsByQId.set(srs.question_id, srs)
  }

  const items: MergedItem[] = []
  for (const q of questions.value) {
    const srs = srsByQId.get(q.id)
    if (!srs) continue

    const subject = subjects.value.find(s => s.id === (q.subjectid || q.subject_id))
    const sourceId = q.sourceid || q.source_id
    const sourceInfo = sourceInfoMap.value.get(sourceId) || {}

    const recallRate = srs.recall_rate ?? 0
    const recallPercent = Math.round(recallRate * 100)
    const n = now()
    const lastAt = srs.last_review_at ?? srs.lastreviewed_at
    const daysSinceLast = lastAt ? Math.max(0, Math.floor((n - lastAt) / 86400)) : -1
    const nextAt = srs.next_review_at
    const daysUntilNext = nextAt ? Math.floor((nextAt - n) / 86400) : null
    const isDue = !nextAt || !daysUntilNext || daysUntilNext <= 0

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
      stabilityText = `${stab.toFixed(1)}`
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
      nextReviewLabel = daysUntilNext !== null ? `${daysUntilNext} 天后` : '待安排'
      lastReviewLabel = daysSinceLast >= 0 ? `${daysSinceLast} 天前` : '未复习'
      stabilityText = `${stab.toFixed(1)}`
    }

    items.push({
      id: q.id,
      questionId: q.id,
      subjectId: q.subjectid || q.subject_id,
      subjectName: subject?.name || '未知',
      prompt: q.prompt || '',
      book: sourceInfo.book || '',
      knowledge: sourceInfo.knowledge || '',
      srs,
      stability: stab,
      difficulty: srs.difficulty ?? 5,
      recallRate,
      nextReviewAt: nextAt,
      lastReviewAt: lastAt,
      reviewCount: srs.review_count ?? 0,
      recallPercent,
      urgencyLabel,
      lastReviewLabel,
      nextReviewLabel,
      stabilityText,
      isDue: isDue,
    })
  }
  return items
})

const filteredItems = computed(() => {
  return mergedItems.value.filter(item => {
    if (filters.value.subject_id && item.subjectId !== filters.value.subject_id) return false
    if (filters.value.book && item.book !== filters.value.book) return false
    if (filters.value.chapter) {
      // Chapter filter not directly on question, apply through source info
      // The chapter info isn't stored on items directly, so skip for now
    }
    if (filters.value.knowledge && item.knowledge !== filters.value.knowledge) return false
    return true
  })
})

const dueList = computed(() => {
  return filteredItems.value
    .filter(item => {
      return item.isDue
    })
    .sort((a, b) => a.recallRate - b.recallRate)
})

const notDueList = computed(() => {
  return filteredItems.value
    .filter(item => {
      return !item.isDue
    })
    .sort((a, b) => (a.nextReviewAt ?? Infinity) - (b.nextReviewAt ?? Infinity))
})

const allFiltered = computed(() => [...dueList.value, ...notDueList.value])

// ============ Methods ============
function getSubjectStyle(subjectId: string) {
  const subject = subjects.value.find(s => s.id === subjectId)
  if (subject?.color) {
    return { backgroundColor: `${subject.color}20`, color: subject.color }
  }
  return { backgroundColor: '#e3f2fd', color: '#1976d2' }
}

function truncate(text: string, len: number) {
  return (text || '').length > len ? text.substring(0, len) + '…' : text
}

const renderMarkdown = (content: string) => {
  if (!content) return ''
  const normalized = content
    .replace(/\\\[/g, '$$')
    .replace(/\\\]/g, '$$')
    .replace(/\\\(/g, '$')
    .replace(/\\\)/g, '$')
  return marked.parse(normalized, { breaks: true, gfm: true }) as string
}

// Cascade filter handlers
function toggleSubjectDropdown() {
  subjectDropdownVisible.value = !subjectDropdownVisible.value
  if (!subjectDropdownVisible.value) closeCascadeWindow()
}

function handleSubjectClick(id: string) {
  filters.value.subject_id = id
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  currentSubjectId.value = id
  currentBook.value = null
  currentChapter.value = null
  chapters.value = []
  knowledges.value = []
  cascadeVisible.value = true
  if (id) {
    getBooks(id).then(b => { books.value = b }).catch(() => { books.value = [] })
  } else {
    books.value = []
  }
}

function handleBookClick(book: string) {
  filters.value.book = book
  filters.value.chapter = ''
  filters.value.knowledge = ''
  currentBook.value = book || null
  currentChapter.value = null
  knowledges.value = []
  if (filters.value.subject_id && book) {
    getChapters(book, filters.value.subject_id).then(c => { chapters.value = c }).catch(() => { chapters.value = [] })
  } else {
    chapters.value = []
  }
}

function handleChapterClick(ch: string) {
  filters.value.chapter = ch
  filters.value.knowledge = ''
  currentChapter.value = ch || null
  if (filters.value.subject_id && filters.value.book && ch) {
    getKnowledges(filters.value.book, ch, filters.value.subject_id).then(k => { knowledges.value = k }).catch(() => { knowledges.value = [] })
  } else {
    knowledges.value = []
  }
}

function selectKnowledge(k: string) {
  filters.value.knowledge = k
  closeCascadeWindow()
}

function selectSubject(id: string) {
  filters.value.subject_id = id
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
  if (id) {
    currentSubjectId.value = id
    cascadeVisible.value = true
    getBooks(id).then(b => { books.value = b }).catch(() => { books.value = [] })
  } else {
    closeCascadeWindow()
  }
}

function closeCascadeWindow() {
  subjectDropdownVisible.value = false
  cascadeVisible.value = false
}

function removeFilter(key: string) {
  if (key === 'subject_id') {
    filters.value.subject_id = ''
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
  filters.value.subject_id = ''
  filters.value.book = ''
  filters.value.chapter = ''
  filters.value.knowledge = ''
}

function buildReviewCard(item: MergedItem): any {
  return {
    questionId: item.questionId,
    srs: item.srs,
    question: questions.value.find(q => q.id === item.questionId) || {},
    subjectName: item.subjectName,
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
onMounted(async () => {
  try {
    const [subs, qs, srs, tags, srcs] = await Promise.all([
      getSubjects(),
      getQuestions(),
      getAllSRSStatus(),
      getFullErrorTags(),
      getSources(),
    ])
    subjects.value = subs
    questions.value = qs as any[]
    srsCards.value = srs as any[]

    const tagMap = new Map<string, string[]>()
    ;(tags as any[]).forEach((tag: any) => {
      // 过滤掉已删除的标签
      if (tag.name.startsWith('[已删除]')) return
      if (!tagMap.has(tag.question_id)) tagMap.set(tag.question_id, [])
      tagMap.get(tag.question_id)!.push(tag.name)
    })
    questionTagsMap.value = tagMap

    const sourceMap = new Map<string, any>()
    ;(srcs as any[]).forEach((src: any) => {
      sourceMap.set(src.id, { book: src.book || '', chapter: src.chapter || '', knowledge: src.knowledge || '' })
    })
    sourceInfoMap.value = sourceMap
  } catch (e) {
    console.error('Preview load failed:', e)
  }
})
</script>

<style scoped>
.preview-page {
  padding: 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
}

/* ===== Filter Bar ===== */
.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.filter-select-wrapper {
  position: relative;
  flex: 1;
  min-width: 120px;
  z-index: 100;
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
  transition: border-color 0.2s;
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

/* ===== Cascade Popup ===== */
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
  width: fit-content;
  max-height: 500px;
  animation: cascadeSlideDown 0.2s ease-out;
  overflow: hidden;
}

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
  z-index: 10;
  padding: 0;
}
.cascade-close-btn:hover {
  background: #ff4d4f;
  color: white;
}

@keyframes cascadeSlideDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.cascade-column {
  flex: 0 0 160px;
  width: 160px;
  border-right: 1px solid var(--border-color);
  padding: 8px 0;
  max-height: 500px;
  overflow-y: auto;
}
.cascade-column:first-child { padding-left: 8px; }
.cascade-column:last-child { border-right: none; padding-right: 40px; }

.column-title {
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 4px;
}

.cascade-item {
  padding: 10px 12px;
  font-size: 14px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.cascade-item:hover,
.cascade-item.active {
  background: var(--primary-color);
  color: white;
}
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

/* ===== Active Filters ===== */
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
}
.filter-tag-close {
  background: none;
  border: none;
  color: white;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
  opacity: 0.8;
}
.filter-tag-close:hover { opacity: 1; }
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

/* ===== Section Header ===== */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.section-header h3 {
  font-size: 16px;
  margin: 0;
  color: var(--text-primary);
}
.section-count {
  font-size: 13px;
  color: var(--text-secondary);
}

/* ===== Card List ===== */
.card-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
}

.error-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 14px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: all 0.2s;
}
.error-card:active {
  transform: scale(0.98);
}

.error-header {
  display: flex;
  gap: 6px;
  margin-bottom: 10px;
  align-items: center;
  flex-wrap: wrap;
}

.subject-tag {
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.source-tag {
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}
.knowledge-tag {
  background: #e0f2f1;
  color: #00796b;
}

.urgency-badge {
  margin-left: auto;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.error-content {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 10px;
  line-height: 1.5;
}

.error-content.markdown-body {
  overflow: hidden;
  max-height: 100%;
}

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
  background: #0f172a;
  padding: 10px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.error-content.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  color: #e2e8f0;
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

.error-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}
.meta-item {
  color: var(--text-secondary);
}

/* ===== Section Divider ===== */
.section-divider {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 20px 0 16px;
  color: var(--text-secondary);
  font-size: 13px;
}
.section-divider::before,
.section-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border-color);
}

/* ===== Empty State ===== */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}
.empty-icon { font-size: 64px; margin-bottom: 16px; }
.empty-state p { font-size: 16px; margin: 0; }

/* ===== FAB ===== */
.fab {
  position: fixed;
  bottom: 80px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 28px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 4px 16px rgba(25, 118, 210, 0.4);
  transition: all 0.2s;
  z-index: 50;
}
.fab:active {
  transform: scale(0.95);
  box-shadow: 0 2px 8px rgba(25, 118, 210, 0.3);
}
.fab-icon { font-size: 18px; }
.fab-badge {
  background: rgba(255, 255, 255, 0.25);
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 14px;
}
</style>
