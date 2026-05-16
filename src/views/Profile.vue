<template>
  <div class="profile-page">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <div>加载中...</div>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="loadError" class="error-state">
      <svg viewBox="0 0 48 48" fill="none" class="error-icon" stroke="currentColor" stroke-width="4">
        <circle cx="24" cy="24" r="20"/>
        <path d="M24 14v12m0 8v0"/>
      </svg>
      <div class="error-description">{{ loadError }}</div>
      <button class="retry-btn" @click="loadData">重试</button>
    </div>

    <!-- 正常内容 -->
    <template v-else>
      <!-- 概览卡片 -->
      <div class="overview-cards">
        <div class="overview-card">
          <div class="card-icon">📊</div>
          <div class="card-content">
            <div class="card-value">{{ overview.total }}</div>
            <div class="card-label">总题目数</div>
          </div>
        </div>
        <div class="overview-card" @click="$router.push('/review')" style="cursor: pointer">
          <div class="card-icon">⏰</div>
          <div class="card-content">
            <div class="card-value due">{{ overview.dueCount }}</div>
            <div class="card-label">待复习</div>
          </div>
        </div>
        <div class="overview-card">
          <div class="card-icon">🧠</div>
          <div class="card-content">
            <div class="card-value memory">{{ overview.currentMemory }}</div>
            <div class="card-label">目前记忆</div>
          </div>
        </div>
        <div class="overview-card">
          <div class="card-icon">🆕</div>
          <div class="card-content">
            <div class="card-value new">{{ overview.newCards }}</div>
            <div class="card-label">新卡片数</div>
          </div>
        </div>
      </div>

      <!-- SRS 详细指标 -->
      <div class="stats-row">
        <div class="stat-item">
          <div class="stat-label">平均稳定性</div>
          <div class="stat-value">{{ srsStats.avg_stability.toFixed(1) }} 天</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">平均难度</div>
          <div class="stat-value">{{ srsStats.avg_difficulty.toFixed(2) }}</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">累计复习</div>
          <div class="stat-value">{{ srsStats.total_reviews }} 次</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">SRS 卡片</div>
          <div class="stat-value">{{ srsStats.total }}</div>
        </div>
      </div>

      <!-- 科目分布 -->
      <!-- <div class="chart-section">
        <h3>科目分布</h3>
        <div v-if="subjectDistribution.length > 0" class="distribution-body">
          <div class="stacked-bar">
            <div
              v-for="(item, index) in subjectDistribution"
              :key="index"
              class="bar-segment"
              :style="{
                width: item.percent + '%',
                background: colors[index % colors.length]
              }"
            ></div>
          </div>
          <div class="bar-total">{{ overview.total }} 题</div>
          <div class="legend">
            <div v-for="(item, index) in subjectDistribution" :key="index" class="legend-item">
              <span class="legend-color" :style="{ background: colors[index % colors.length] }"></span>
              <span class="legend-label">{{ item.name }}</span>
              <span class="legend-value">{{ item.count }}题 ({{ item.percent }}%)</span>
            </div>
          </div>
        </div>
        <div v-else class="empty-state">
          <div class="empty-description">暂无科目数据</div>
        </div>
      </div> -->

      <!-- 科目 × 难度热力图 -->
      <div class="chart-section">
        <h3>科目 × 难度热力图</h3>
        <div class="heatmap-hint">颜色越深表示该区间题目越多，难度区间动态10等分</div>
        <div v-if="heatmapData.length > 0" class="heatmap-wrapper" @click.self="selectedCell = null">
          <table class="heatmap-table">
            <thead>
              <tr>
                <th class="row-header">科目</th>
                <th v-for="b in 10" :key="b" class="col-header">{{ b }}</th>
                <th class="row-header total-col">合计</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, ri) in heatmapData" :key="ri">
                <td class="row-label" :title="row.subject">{{ row.subjectShort }}</td>
                <td
                  v-for="(cell, ci) in row.buckets"
                  :key="ci"
                  class="heat-cell"
                  :class="{ selected: selectedCell?.ri === ri && selectedCell?.ci === ci }"
                  :style="getHeatStyle(cell.count, heatmapMax)"
                  @click="toggleCell(ri, ci)"
                >{{ cell.count || '' }}</td>
                <td class="row-total">{{ row.total }}</td>
              </tr>
            </tbody>
          </table>

          <!-- 点击详情弹窗 -->
          <div v-if="selectedCell" class="cell-detail">
            <div class="cell-detail-row">科目: {{ selectedCell.subject }}</div>
            <div class="cell-detail-row">难度区间: {{ selectedCell.label }}</div>
            <div class="cell-detail-row">题目数量: <strong>{{ selectedCell.count }}</strong></div>
          </div>
          <div class="heatmap-legend">
            <span>少</span>
            <span v-for="l in 5" :key="l" class="legend-block" :style="{ background: `rgba(33, 150, 243, ${0.1 + l * 0.18})` }"></span>
            <span>多</span>
          </div>
          <div class="difficulty-range" v-if="difficultyRange">
            难度范围: {{ difficultyRange.min.toFixed(4) }} ~ {{ difficultyRange.max.toFixed(4) }}
          </div>
        </div>
        <div v-else class="empty-state">
          <div class="empty-description">暂无足够数据生成热力图</div>
        </div>
      </div>

      <!-- 复习间隔分布 -->
      <div class="chart-section">
        <h3>复习间隔分布</h3>
        <div v-if="intervalBuckets.length > 0" class="interval-chart">
          <div class="interval-bars">
            <div v-for="(bucket, index) in intervalBuckets" :key="index" class="interval-bar-group">
              <div class="bar-value">{{ bucket.count }}</div>
              <div class="bar-fill" :style="{ height: getIntervalBarHeight(bucket.count) + '%', background: bucket.color }"></div>
              <div class="bar-label">{{ bucket.label }}</div>
            </div>
          </div>
        </div>
        <div v-else class="empty-state">
          <div class="empty-description">暂无间隔数据</div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { getQuestionStats, getQuestions } from '../apis/errorQuestions'
import { getDueCount, getSRSStatistics, getAllSRSStatus } from '../apis/srs'
import { getSubjects } from '../apis/subjects'
import type { Subject } from '../types'

// ==================== 状态 ====================
const loading = ref(true)
const loadError = ref('')

// 原始数据
const questionTotal = ref(0)
const dueCount = ref(0)
const srsStats = ref({ total: 0, due_count: 0, new_cards: 0, avg_stability: 0, avg_difficulty: 0, total_reviews: 0 })
const subjects = ref<Subject[]>([])
const questions = ref<{ id: string; subject_id: string }[]>([])
const allCards = ref<{ question_id: string; difficulty: number; next_review_at: number | null; review_count: number }[]>([])

const colors = ['#1976d2', '#e65100', '#7b1fa2', '#43a047', '#00bcd4', '#ff9800', '#795548', '#607d8b', '#c62828', '#283593']

// 难度区间标注
const difficultyRange = ref<{ min: number; max: number } | null>(null)

// ==================== 概览卡片 ====================
const overview = computed(() => ({
  total: questionTotal.value,
  dueCount: dueCount.value,
  currentMemory: Math.max(0, questionTotal.value - dueCount.value),
  newCards: srsStats.value.new_cards
}))

// ==================== 科目分布 ====================
interface SubjectDistItem {
  name: string
  count: number
  percent: number
  color: string
}

const subjectDistribution = computed<SubjectDistItem[]>(() => {
  const subjectMap = new Map<string, string>()
  subjects.value.forEach(s => subjectMap.set(s.id, s.name))

  const countMap = new Map<string, number>()
  questions.value.forEach(q => {
    const sid = q.subject_id
    countMap.set(sid, (countMap.get(sid) || 0) + 1)
  })

  const total = questionTotal.value
  if (total === 0) return []

  return Array.from(countMap.entries())
    .map(([id, count]) => ({
      name: subjectMap.get(id) || '未分类',
      count,
      percent: Math.round((count / total) * 100),
      color: ''
    }))
    .sort((a, b) => b.count - a.count)
})

// ==================== 科目 × 难度热力图 ====================

interface HeatmapBucket {
  count: number
  label: string
  rangeMin: number
  rangeMax: number
}

interface HeatmapRow {
  subject: string
  subjectShort: string
  buckets: HeatmapBucket[]
  total: number
}

const heatmapData = computed<HeatmapRow[]>(() => {
  if (allCards.value.length === 0 || subjects.value.length === 0) return []

  // 建立 question_id → subject_id 映射
  const qToSubject = new Map<string, string>()
  questions.value.forEach(q => qToSubject.set(q.id, q.subject_id))

  // 建立 subject_id → name 映射
  const subjectNameMap = new Map<string, string>()
  subjects.value.forEach(s => subjectNameMap.set(s.id, s.name))

  // 收集有 subject 关联的卡片，收集难度值
  const validCards: { subjectId: string; difficulty: number }[] = []
  const difficulties: number[] = []

  for (const card of allCards.value) {
    const subjectId = qToSubject.get(card.question_id)
    if (!subjectId) continue
    validCards.push({ subjectId, difficulty: card.difficulty })
    difficulties.push(card.difficulty)
  }

  if (validCards.length === 0 || difficulties.length === 0) return []

  const minDiff = Math.min(...difficulties)
  const maxDiff = Math.max(...difficulties)
  difficultyRange.value = { min: minDiff, max: maxDiff }

  const range = maxDiff - minDiff
  const bucketSize = range / 10

  // 按 subject 分组
  const subjectGroups = new Map<string, number[]>()
  for (const vc of validCards) {
    const arr = subjectGroups.get(vc.subjectId) || []
    arr.push(vc.difficulty)
    subjectGroups.set(vc.subjectId, arr)
  }

  // 构建矩阵
  const rows: HeatmapRow[] = []

  for (const [subjId, diffs] of subjectGroups) {
    const buckets: HeatmapBucket[] = []
    for (let i = 0; i < 10; i++) {
      const lo = minDiff + i * bucketSize
      const hi = i === 9 ? maxDiff + 0.001 : minDiff + (i + 1) * bucketSize
      const count = diffs.filter(d => d >= lo && d < hi).length
      buckets.push({
        count,
        label: `${(lo).toFixed(4)}-${i === 9 ? maxDiff.toFixed(4) : (minDiff + (i + 1) * bucketSize).toFixed(4)}`,
        rangeMin: lo,
        rangeMax: hi
      })
    }
    const name = subjectNameMap.get(subjId) || '未分类'
    rows.push({
      subject: name,
      subjectShort: name.length > 4 ? name.slice(0, 4) + '…' : name,
      buckets,
      total: diffs.length
    })
  }

  rows.sort((a, b) => b.total - a.total)
  return rows
})

const heatmapMax = computed(() => {
  let max = 0
  for (const row of heatmapData.value) {
    for (const b of row.buckets) {
      if (b.count > max) max = b.count
    }
  }
  return max
})

// ==================== 热力图点击详情 ====================
const selectedCell = ref<{
  ri: number; ci: number;
  subject: string; label: string; count: number
} | null>(null)

function toggleCell(ri: number, ci: number) {
  const row = heatmapData.value[ri]
  if (!row) return
  const cell = row.buckets[ci]
  if (!cell) return

  if (selectedCell.value?.ri === ri && selectedCell.value?.ci === ci) {
    selectedCell.value = null
  } else {
    selectedCell.value = { ri, ci, subject: row.subject, label: cell.label, count: cell.count }
  }
}

const getHeatStyle = (count: number, max: number) => {
  if (count === 0 || max === 0) return { background: 'transparent' }
  const intensity = Math.min(1, count / max)
  return { background: `rgba(33, 150, 243, ${0.08 + intensity * 0.82})` }
}

// ==================== 复习间隔分布 ====================
interface IntervalBucket {
  label: string
  count: number
  color: string
}

const intervalBuckets = computed<IntervalBucket[]>(() => {
  const now = Math.floor(Date.now() / 1000)

  const buckets: IntervalBucket[] = [
    { label: '已到期', count: 0, color: '#ef5350' },
    { label: '1-3天', count: 0, color: '#ff9800' },
    { label: '4-7天', count: 0, color: '#ffc107' },
    { label: '8-14天', count: 0, color: '#66bb6a' },
    { label: '15-30天', count: 0, color: '#42a5f5' },
    { label: '30天+', count: 0, color: '#7e57c2' },
    { label: '新卡片', count: 0, color: '#bdbdbd' },
  ]

  for (const card of allCards.value) {
    if (card.review_count === 0 || card.next_review_at === null) {
      buckets[6].count++ // 新卡片
      continue
    }
    const daysUntil = (card.next_review_at - now) / 86400
    if (daysUntil <= 0) buckets[0].count++
    else if (daysUntil <= 3) buckets[1].count++
    else if (daysUntil <= 7) buckets[2].count++
    else if (daysUntil <= 14) buckets[3].count++
    else if (daysUntil <= 30) buckets[4].count++
    else buckets[5].count++
  }

  return buckets
})

const intervalMaxCount = computed(() => Math.max(...intervalBuckets.value.map(b => b.count), 1))

const getIntervalBarHeight = (count: number) => (count / intervalMaxCount.value) * 100

// ==================== 数据加载 ====================
async function loadData() {
  loading.value = true
  loadError.value = ''

  try {
    const [statsRes, dueRes, srsRes, subjRes, qsRes, cardsRes] = await Promise.all([
      getQuestionStats().catch(() => ({ total: 0 })),
      getDueCount().catch(() => 0),
      getSRSStatistics().catch(() => ({ total: 0, due_count: 0, new_cards: 0, avg_stability: 0, avg_difficulty: 0, total_reviews: 0 })),
      getSubjects().catch(() => [] as Subject[]),
      getQuestions().catch(() => []),
      getAllSRSStatus().catch(() => []),
    ])

    questionTotal.value = statsRes.total
    dueCount.value = dueRes
    srsStats.value = srsRes
    subjects.value = subjRes
    // 注意: 后端返回的字段名是 subjectid（非 subject_id）
    questions.value = qsRes.map((q: any) => ({ id: q.id, subject_id: q.subjectid }))
    allCards.value = cardsRes.map((c: any) => ({
      question_id: c.question_id,
      difficulty: c.difficulty,
      next_review_at: c.next_review_at,
      review_count: c.review_count
    }))

    // 调试: SRS 难度分布
    if (cardsRes.length > 0) {
      const difficulties = cardsRes.map((c: any) => c.difficulty)
      console.log('[Profile] SRS 难度原始值:', difficulties.slice(0, 20))
      console.log('[Profile] SRS 难度去重:', [...new Set(difficulties)])
      const unique: any[] = [...new Set(difficulties)]
      console.log('[Profile] SRS 难度 min/max:', { min: Math.min(...unique), max: Math.max(...unique) })
    }

    // 调试: 科目映射是否正确
    if (subjRes.length > 0 && qsRes.length > 0) {
      const sample: any = qsRes[0]
      console.log('[Profile] 字段检查:', { subjectid: sample.subjectid, subject_id: sample.subject_id, id: sample.id })
      console.log('[Profile] 科目列表:', subjRes)
    }
  } catch (e: any) {
    loadError.value = e?.toString() || '加载失败'
  } finally {
    loading.value = false
  }
}

loadData()
</script>

<style scoped>
.profile-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* ========== 加载 & 错误 ========== */
.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  gap: 16px;
  color: var(--text-secondary);
  font-size: 14px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon {
  width: 48px;
  height: 48px;
  color: #ef5350;
  opacity: 0.7;
}

.error-description {
  color: var(--text-secondary);
  font-size: 14px;
  text-align: center;
}

.retry-btn {
  padding: 8px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

/* ========== 概览卡片 ========== */
.overview-cards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.overview-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  transition: transform 0.2s;
}

.overview-card:active {
  transform: scale(0.97);
}

.card-icon { font-size: 32px; }
.card-content { flex: 1; }
.card-value { font-size: 24px; font-weight: bold; color: var(--primary-color); }
.card-value.due { color: #ef5350; }
.card-value.memory { color: #66bb6a; }
.card-value.new { color: #ab47bc; }
.card-label { font-size: 12px; color: var(--text-secondary); margin-top: 2px; }

/* ========== SRS 详细指标 ========== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.stat-item {
  background: var(--card-bg);
  border-radius: 8px;
  padding: 12px 8px;
  text-align: center;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}

.stat-label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

/* ========== 图表公用 ========== */
.chart-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.chart-section h3 {
  font-size: 16px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 0;
  color: var(--text-secondary);
  font-size: 14px;
}

/* ========== 科目分布 ========== */
.distribution-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stacked-bar {
  display: flex;
  height: 24px;
  border-radius: 12px;
  overflow: hidden;
}

.bar-segment {
  transition: width 0.3s;
  min-width: 2px;
}

.bar-total {
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
}

.legend {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
  flex-shrink: 0;
}

.legend-label { flex: 1; color: var(--text-primary); }
.legend-value { color: var(--text-secondary); }

/* ========== 热力图 ========== */
.heatmap-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.heatmap-wrapper {
  overflow-x: auto;
}

.heatmap-table {
  border-collapse: collapse;
  width: 100%;
  min-width: 480px;
  font-size: 12px;
}

.heatmap-table th,
.heatmap-table td {
  text-align: center;
  padding: 6px 4px;
  border: 1px solid var(--border-color);
}

.row-header {
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-weight: 600;
  white-space: nowrap;
}

.col-header {
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 11px;
  width: 36px;
}

.total-col {
  font-size: 11px;
}

.row-label {
  text-align: left;
  padding: 6px 8px !important;
  color: var(--text-primary);
  font-weight: 500;
  white-space: nowrap;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.heat-cell {
  transition: background 0.2s;
  color: var(--text-primary);
  font-size: 11px;
}

.row-total {
  font-weight: 600;
  color: var(--text-primary);
  background: var(--bg-secondary);
}

.heatmap-legend {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
  margin-top: 8px;
  font-size: 11px;
  color: var(--text-secondary);
}

.legend-block {
  width: 16px;
  height: 12px;
  border-radius: 2px;
}

.difficulty-range {
  font-size: 11px;
  color: var(--text-secondary);
  text-align: right;
  margin-top: 4px;
}

/* 热力图点击详情 */
.cell-detail {
  margin-top: 12px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.8;
}

.cell-detail-row {
  color: var(--text-primary);
}

.cell-detail-row strong {
  color: var(--primary-color);
  font-size: 18px;
}

.heat-cell.selected {
  outline: 2px solid var(--primary-color);
  outline-offset: -1px;
}

/* ========== 复习间隔分布 ========== */
.interval-chart {
  padding: 12px 0;
}

.interval-bars {
  display: flex;
  justify-content: space-around;
  align-items: flex-end;
  height: 140px;
  gap: 6px;
}

.interval-bar-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
  justify-content: flex-end;
}

.bar-fill {
  width: 70%;
  max-width: 40px;
  border-radius: 4px 4px 0 0;
  min-height: 4px;
  transition: height 0.3s;
}

.bar-value {
  font-size: 12px;
  color: var(--text-primary);
  margin-bottom: 4px;
  font-weight: 500;
}

.bar-label {
  font-size: 10px;
  color: var(--text-secondary);
  margin-top: 6px;
  text-align: center;
}

/* ========== 响应式 ========== */
@media (max-width: 480px) {
  .overview-cards {
    gap: 8px;
  }
  .card-value { font-size: 20px; }
  .legend { grid-template-columns: 1fr; }
  .bar-label { font-size: 9px; }
}
</style>
