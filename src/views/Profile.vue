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

    <!-- 缩略统计页面 -->
    <div class="stats-preview" @click="goToStats">
      <div class="stats-header">
        <h3>学习统计</h3>
        <span class="expand-icon">→</span>
      </div>
      <div class="stats-overview">
        <div class="stats-item">
          <div class="stats-value">{{ overview.total }}</div>
          <div class="stats-label">总错题数</div>
        </div>
        <div class="stats-item">
          <div class="stats-value">{{ overview.mastered }}</div>
          <div class="stats-label">已掌握</div>
        </div>
        <div class="stats-item">
          <div class="stats-value">{{ overview.accuracy }}%</div>
          <div class="stats-label">正确率</div>
        </div>
        <div class="stats-item">
          <div class="stats-value">{{ overview.streak }}</div>
          <div class="stats-label">连续学习</div>
        </div>
      </div>
      <div class="stats-trend">
        <div class="trend-header">本周学习</div>
        <div class="trend-preview">
          <div v-for="(item, index) in weeklyStats" :key="index" class="trend-bar-preview">
            <div class="bar-fill-preview" :style="{ height: getBarHeight(item.count) + '%' }"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 设置选项 -->
    <div class="settings-section">
      <h3>设置选项</h3>
      
      <!-- 主题设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">🎨</div>
          <div class="setting-name">主题设置</div>
        </div>
        <div class="setting-action">
          <select v-model="theme" class="theme-select">
            <option value="light">浅色主题</option>
            <option value="dark">深色主题</option>
            <option value="system">跟随系统</option>
          </select>
        </div>
      </div>

      <!-- AI 选项 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">🤖</div>
          <div class="setting-name">AI 辅助</div>
        </div>
        <div class="setting-action">
          <div class="toggle-switch">
            <input type="checkbox" v-model="aiEnabled" id="aiToggle" />
            <label for="aiToggle" class="toggle-label"></label>
          </div>
        </div>
      </div>

      <!-- AI 选项详情 -->
      <div v-if="aiEnabled" class="ai-options">
        <div class="setting-item nested">
          <div class="setting-info">
            <div class="setting-icon">📝</div>
            <div class="setting-name">AI 错题解析</div>
          </div>
          <div class="setting-action">
            <div class="toggle-switch">
              <input type="checkbox" v-model="aiAnalysis" id="aiAnalysisToggle" />
              <label for="aiAnalysisToggle" class="toggle-label"></label>
            </div>
          </div>
        </div>

        <div class="setting-item nested">
          <div class="setting-info">
            <div class="setting-icon">📚</div>
            <div class="setting-name">AI 智能复习</div>
          </div>
          <div class="setting-action">
            <div class="toggle-switch">
              <input type="checkbox" v-model="aiReview" id="aiReviewToggle" />
              <label for="aiReviewToggle" class="toggle-label"></label>
            </div>
          </div>
        </div>

        <div class="setting-item nested">
          <div class="setting-info">
            <div class="setting-icon">🔍</div>
            <div class="setting-name">AI 相似题型推荐</div>
          </div>
          <div class="setting-action">
            <div class="toggle-switch">
              <input type="checkbox" v-model="aiRecommend" id="aiRecommendToggle" />
              <label for="aiRecommendToggle" class="toggle-label"></label>
            </div>
          </div>
        </div>
      </div>

      <!-- 其他设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">📱</div>
          <div class="setting-name">设备管理</div>
        </div>
        <div class="setting-action">
          <span class="arrow-icon">→</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">🔔</div>
          <div class="setting-name">通知设置</div>
        </div>
        <div class="setting-action">
          <span class="arrow-icon">→</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">ℹ️</div>
          <div class="setting-name">关于应用</div>
        </div>
        <div class="setting-action">
          <span class="arrow-icon">→</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

// 用户信息
const userInfo = ref({
  nickname: '智能错题本用户',
  studentId: '202501001',
  avatar: 'https://via.placeholder.com/100'
})

// 统计数据
const overview = ref({
  total: 128,
  mastered: 89,
  accuracy: 72,
  streak: 7
})

const weeklyStats = ref([
  { day: '周一', count: 12 },
  { day: '周二', count: 18 },
  { day: '周三', count: 15 },
  { day: '周四', count: 22 },
  { day: '周五', count: 8 },
  { day: '周六', count: 25 },
  { day: '周日', count: 20 }
])

// 主题设置
const theme = ref('light')

// AI 选项
const aiEnabled = ref(true)
const aiAnalysis = ref(true)
const aiReview = ref(true)
const aiRecommend = ref(false)

// 计算柱状图高度
const getBarHeight = (count: number) => {
  const max = Math.max(...weeklyStats.value.map(item => item.count))
  return (count / max) * 100
}

// 跳转到完整统计页面
const goToStats = () => {
  router.push('/stats-full')
}

// 主题切换
const handleThemeChange = () => {
  // 实际应用中这里会调用主题切换逻辑
  console.log('主题切换为:', theme.value)
}

// AI 选项切换
const handleAiToggle = () => {
  console.log('AI', aiEnabled.value ? '开启' : '关闭')
}
</script>

<style scoped>
.profile-page {
  padding: 20px;
  padding-bottom: 80px;
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

/* 缩略统计页面 */
.stats-preview {
  background: var(--card-bg);
  border-radius: 16px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.stats-preview:active {
  transform: scale(0.98);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.stats-header h3 {
  font-size: 18px;
  margin: 0;
  color: var(--text-primary);
}

.expand-icon {
  font-size: 16px;
  color: var(--text-secondary);
}

.stats-overview {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.stats-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stats-value {
  font-size: 18px;
  font-weight: bold;
  color: var(--primary-color);
  margin-bottom: 4px;
}

.stats-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.stats-trend {
  margin-top: 16px;
}

.trend-header {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.trend-preview {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  height: 80px;
  gap: 6px;
  padding: 0 4px;
}

.trend-bar-preview {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.bar-fill-preview {
  width: 100%;
  background: linear-gradient(180deg, var(--primary-color), #42a5f5);
  border-radius: 4px 4px 0 0;
  min-height: 4px;
  transition: height 0.3s;
}

/* 设置选项 */
.settings-section {
  background: var(--card-bg);
  border-radius: 16px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.settings-section h3 {
  font-size: 18px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid var(--border-color);
  transition: background 0.2s;
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-item:hover {
  background: var(--input-bg);
  border-radius: 8px;
  padding: 16px;
  margin: 0 -16px;
}

.setting-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-icon {
  font-size: 20px;
}

.setting-name {
  font-size: 16px;
  color: var(--text-primary);
}

.setting-action {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 主题选择器 */
.theme-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.theme-select:hover {
  border-color: var(--primary-color);
}

/* 开关按钮 */
.toggle-switch {
  position: relative;
  width: 50px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-label {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border-color);
  transition: 0.4s;
  border-radius: 24px;
}

.toggle-label:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.4s;
  border-radius: 50%;
}

input:checked + .toggle-label {
  background-color: var(--primary-color);
}

input:checked + .toggle-label:before {
  transform: translateX(26px);
}

/* AI 嵌套选项 */
.ai-options {
  margin-left: 32px;
  margin-top: 8px;
  padding-top: 8px;
  border-left: 2px solid var(--border-color);
}

.setting-item.nested {
  padding: 12px 0;
  margin-left: 16px;
}

.setting-item.nested:hover {
  margin: 0 -16px 0 0;
  padding: 12px 16px;
}

/* 箭头图标 */
.arrow-icon {
  font-size: 18px;
  color: var(--text-secondary);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .user-card {
    flex-direction: column;
    text-align: center;
  }

  .stats-overview {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .setting-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .setting-action {
    align-self: stretch;
  }

  .theme-select {
    width: 100%;
  }
}
</style>