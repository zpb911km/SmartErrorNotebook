<template>
  <div class="stats-page">
    <div class="overview-cards">
      <div class="overview-card">
        <div class="card-icon">📊</div>
        <div class="card-content">
          <div class="card-value">{{ overview.total }}</div>
          <div class="card-label">总错题数</div>
        </div>
      </div>
      <div class="overview-card">
        <div class="card-icon">✅</div>
        <div class="card-content">
          <div class="card-value">{{ overview.mastered }}</div>
          <div class="card-label">已掌握</div>
        </div>
      </div>
      <div class="overview-card">
        <div class="card-icon">📈</div>
        <div class="card-content">
          <div class="card-value">{{ overview.accuracy }}%</div>
          <div class="card-label">正确率</div>
        </div>
      </div>
      <div class="overview-card">
        <div class="card-icon">🔥</div>
        <div class="card-content">
          <div class="card-value">{{ overview.streak }}</div>
          <div class="card-label">连续学习</div>
        </div>
      </div>
    </div>

    <div class="chart-section">
      <h3>科目分布</h3>
      <div class="chart-placeholder">
        <div class="pie-chart">
          <div v-for="(item, index) in subjectDistribution" :key="index" class="pie-segment" :style="getPieStyle(item, index)">
            <span class="pie-label">{{ item.subject }}</span>
          </div>
        </div>
        <div class="legend">
          <div v-for="(item, index) in subjectDistribution" :key="index" class="legend-item">
            <span class="legend-color" :style="{ background: colors[index] }"></span>
            <span class="legend-label">{{ item.subjectName }}</span>
            <span class="legend-value">{{ item.count }}题</span>
          </div>
        </div>
      </div>
    </div>

    <div class="chart-section">
      <h3>学习趋势</h3>
      <div class="trend-chart">
        <div class="trend-bars">
          <div v-for="(item, index) in weeklyStats" :key="index" class="trend-bar">
            <div class="bar-fill" :style="{ height: getBarHeight(item.count) + '%' }"></div>
            <div class="bar-label">{{ item.day }}</div>
            <div class="bar-value">{{ item.count }}</div>
          </div>
        </div>
      </div>
    </div>

    <div class="weak-section">
      <h3>薄弱环节</h3>
      <div class="weak-list">
        <div v-for="(item, index) in weakPoints" :key="index" class="weak-item">
          <div class="weak-rank">{{ index + 1 }}</div>
          <div class="weak-info">
            <div class="weak-name">{{ item.name }}</div>
            <div class="weak-count">{{ item.count }} 次错误</div>
          </div>
          <div class="weak-action" @click="reviewWeak(item)">
            <span>复习</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const overview = ref({
  total: 128,
  mastered: 89,
  accuracy: 72,
  streak: 7
})

const subjectDistribution = ref([
  { subject: 'math', subjectName: '数学', count: 45 },
  { subject: 'physics', subjectName: '物理', count: 32 },
  { subject: 'chemistry', subjectName: '化学', count: 28 },
  { subject: 'english', subjectName: '英语', count: 23 }
])

const colors = ['#1976d2', '#e65100', '#7b1fa2', '#43a047']

const weeklyStats = ref([
  { day: '周一', count: 12 },
  { day: '周二', count: 18 },
  { day: '周三', count: 15 },
  { day: '周四', count: 22 },
  { day: '周五', count: 8 },
  { day: '周六', count: 25 },
  { day: '周日', count: 20 }
])

const weakPoints = ref([
  { name: '函数与导数', count: 15 },
  { name: '牛顿运动定律', count: 12 },
  { name: '化学方程式配平', count: 10 },
  { name: '英语语法', count: 8 }
])

const getPieStyle = (item: any, index: number) => {
  const total = subjectDistribution.value.reduce((sum, i) => sum + i.count, 0)
  const percent = (item.count / total) * 100
  return {
    background: colors[index],
    width: percent + '%'
  }
}

const getBarHeight = (count: number) => {
  const max = Math.max(...weeklyStats.value.map(item => item.count))
  return (count / max) * 100
}

const reviewWeak = (item: any) => {
  console.log('复习薄弱点', item)
}
</script>

<style scoped>
.stats-page {
  padding: 20px;
  padding-bottom: 80px;
}

.overview-cards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}

.overview-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.card-icon {
  font-size: 32px;
}

.card-content {
  flex: 1;
}

.card-value {
  font-size: 24px;
  font-weight: bold;
  color: var(--primary-color);
}

.card-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.chart-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.chart-section h3 {
  font-size: 16px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.chart-placeholder {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.pie-chart {
  display: flex;
  height: 20px;
  border-radius: 10px;
  overflow: hidden;
}

.pie-segment {
  display: flex;
  align-items: center;
  justify-content: center;
  transition: width 0.3s;
}

.pie-label {
  color: white;
  font-size: 10px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
}

.legend-label {
  flex: 1;
  color: var(--text-primary);
}

.legend-value {
  color: var(--text-secondary);
}

.trend-chart {
  padding: 16px 0;
}

.trend-bars {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  height: 120px;
  gap: 8px;
}

.trend-bar {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
}

.bar-fill {
  width: 100%;
  background: linear-gradient(180deg, var(--primary-color), #42a5f5);
  border-radius: 4px 4px 0 0;
  min-height: 4px;
  transition: height 0.3s;
}

.bar-label {
  font-size: 10px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.bar-value {
  font-size: 12px;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.weak-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.weak-section h3 {
  font-size: 16px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.weak-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.weak-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--input-bg);
  border-radius: 8px;
}

.weak-rank {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: bold;
}

.weak-info {
  flex: 1;
}

.weak-name {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.weak-count {
  font-size: 12px;
  color: var(--text-secondary);
}

.weak-action {
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.3s;
}

.weak-action:active {
  transform: scale(0.98);
}
</style>