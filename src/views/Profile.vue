<template>
  <div class="profile-page">
    <!-- 用户信息卡片 -->
    <div class="user-card">
      <div class="avatar-container">
        <img class="avatar" src="https://via.placeholder.com/100" alt="头像" />
      </div>
      <div class="user-info">
        <div class="nickname">智能错题本用户</div>
        <div class="student-id">学号: 202501001</div>
      </div>
    </div>

    <!-- 详细统计页面 -->
    <div class="stats-section">
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

      <!-- AI接口信息 -->
      <div class="ai-section">
        <div class="ai-card">
          <div class="ai-header">
            <div class="ai-header-content">
              <div class="ai-icon">🤖</div>
              <h3>AI 助手</h3>
            </div>
            <label class="ai-toggle">
              <input type="checkbox" v-model="aiStatus.enabled" @change="toggleAIStatus">
              <span class="toggle-slider"></span>
            </label>
          </div>
          <div class="ai-info">
            <div class="ai-token">
              <span class="token-label">剩余 Token:</span>
              <span class="token-value">{{ aiStatus.remainingTokens }}</span>
              <button class="recharge-btn" @click="openRechargeDialog">充值</button>
            </div>
            <div class="ai-model">
              <span class="model-label">接入 AI:</span>
              <select v-model="aiStatus.modelName" class="model-select" @change="changeModel">
                <option value="GPT-4">GPT-4</option>
                <option value="GPT-3.5 Turbo">GPT-3.5 Turbo</option>
                <option value="Claude 3">Claude 3</option>
                <option value="Gemini Pro">Gemini Pro</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, onMounted } from 'vue'
import { getQuestions } from '../apis/errorQuestions'
import { getSubjects } from '../apis/subjects'

// 统计数据
const overview = ref({
  total: 0,
  mastered: 0,
  accuracy: 0,
  streak: 0
})

const subjectDistribution = ref([])

const colors = ['#1976d2', '#e65100', '#7b1fa2', '#43a047', '#00bcd4', '#ff9800', '#795548', '#607d8b']

const weeklyStats = ref([
  { day: '周一', count: 0 },
  { day: '周二', count: 0 },
  { day: '周三', count: 0 },
  { day: '周四', count: 0 },
  { day: '周五', count: 0 },
  { day: '周六', count: 0 },
  { day: '周日', count: 0 }
])

// 注入全局AI状态
const aiStatus = inject('aiState')

const getPieStyle = (item: any, index: number) => {
  const total = subjectDistribution.value.reduce((sum: number, i: any) => sum + i.count, 0)
  if (total === 0) return { background: colors[index], width: '0%' }
  const percent = (item.count / total) * 100
  return {
    background: colors[index],
    width: percent + '%'
  }
}

const getBarHeight = (count: number) => {
  const max = Math.max(...weeklyStats.value.map(item => item.count))
  if (max === 0) return 0
  return (count / max) * 100
}

// 从数据库获取数据
onMounted(async () => {
  try {
    // 获取总错题数
    const questions = await getQuestions()
    overview.value.total = questions.length
    
    // 模拟已掌握的题目数（实际应该从SRS数据中获取）
    overview.value.mastered = Math.floor(questions.length * 0.7)
    
    // 模拟正确率
    overview.value.accuracy = 72
    
    // 模拟连续学习天数
    overview.value.streak = 7
    
    // 获取科目列表
    const subjects = await getSubjects()
    
    // 模拟科目分布
    subjectDistribution.value = subjects.map((subject: any, index: number) => ({
      subject: subject.name.toLowerCase(),
      subjectName: subject.name,
      count: Math.floor(Math.random() * 50) + 10
    }))
    
    // 模拟每周学习数据
    weeklyStats.value = [
      { day: '周一', count: Math.floor(Math.random() * 30) + 5 },
      { day: '周二', count: Math.floor(Math.random() * 30) + 5 },
      { day: '周三', count: Math.floor(Math.random() * 30) + 5 },
      { day: '周四', count: Math.floor(Math.random() * 30) + 5 },
      { day: '周五', count: Math.floor(Math.random() * 30) + 5 },
      { day: '周六', count: Math.floor(Math.random() * 30) + 5 },
      { day: '周日', count: Math.floor(Math.random() * 30) + 5 }
    ]
  } catch (error) {
    console.error('获取数据失败:', error)
    // 保持默认值为0
  }
})

// AI功能方法
const toggleAIStatus = () => {
  console.log('AI状态已切换为:', aiStatus.value.enabled ? '开启' : '关闭')
  // 这里可以添加实际的状态切换逻辑
}

const openRechargeDialog = () => {
  console.log('打开充值对话框')
  // 这里可以添加实际的充值逻辑
  // 模拟充值操作
  const rechargeAmount = 1000
  aiStatus.value.remainingTokens += rechargeAmount
  console.log(`已充值 ${rechargeAmount} Token，当前余额: ${aiStatus.value.remainingTokens}`)
}

const changeModel = () => {
  console.log('模型已切换为:', aiStatus.value.modelName)
  // 这里可以添加实际的模型切换逻辑
}
</script>

<style scoped>
.profile-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  max-width: 800px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 用户信息卡片 */
.user-card {
  background: var(--card-bg);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  align-items: center;
  gap: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.avatar-container {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  overflow: hidden;
  border: 3px solid var(--primary-color);
}

.avatar {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.user-info {
  flex: 1;
}

.nickname {
  font-size: 20px;
  font-weight: bold;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.student-id {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 统计部分 */
.stats-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

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

.ai-section {
  margin-top: 20px;
}

.ai-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  border-left: 4px solid #9c27b0;
}

.ai-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.ai-header-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.ai-icon {
  font-size: 24px;
}

.ai-header h3 {
  font-size: 16px;
  margin: 0;
  color: var(--text-primary);
}

/* AI开关样式 */
.ai-toggle {
  position: relative;
  display: inline-block;
  width: 50px;
  height: 24px;
}

.ai-toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .4s;
  border-radius: 24px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

input:checked + .toggle-slider {
  background-color: #9c27b0;
}

input:focus + .toggle-slider {
  box-shadow: 0 0 1px #9c27b0;
}

input:checked + .toggle-slider:before {
  transform: translateX(26px);
}

.ai-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.ai-token, .ai-model {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

/* 充值按钮样式 */
.recharge-btn {
  background: #ff9800;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.3s;
}

.recharge-btn:hover {
  background: #f57c00;
  transform: translateY(-1px);
}

/* 模型选择样式 */
.model-select {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 14px;
  cursor: pointer;
  outline: none;
  transition: all 0.3s;
}

.model-select:hover {
  border-color: var(--primary-color);
}

.model-select:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(33, 150, 243, 0.2);
}

.status-label, .token-label, .model-label {
  font-size: 14px;
  color: var(--text-secondary);
}

.status-value, .token-value, .model-value {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.ai-status.active .status-value {
  color: #4caf50;
}

.ai-status:not(.active) .status-value {
  color: #f44336;
}

.token-value {
  color: #ff9800;
}

.model-value {
  color: #2196f3;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .user-card {
    flex-direction: column;
    text-align: center;
  }
}
</style>