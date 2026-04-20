<template>
  <div class="home-page minimal">
    <!-- 顶部区域 -->
  <div class="minimal-header">
      <h1 class="app-title">智能错题本</h1>
    </div>

    <!-- 核心功能区 -->
    <div class="core-actions">
      <button class="primary-action" @click="$router.push('/add')">
        <span class="action-icon">➕</span>
        <span class="action-text">添加错题</span>
      </button>
      <div class="secondary-actions">
        <button class="secondary-action" @click="$router.push('/review')">
          <span class="action-icon">📖</span>
          <span class="action-text">开始复习</span>
        </button>
        <button class="secondary-action" @click="$router.push('/manage')">
          <span class="action-icon">📋</span>
          <span class="action-text">错题管理</span>
        </button>
      </div>
    </div>

    <!-- 学习概览 -->
    <div class="overview-section">
      <h2 class="section-title">学习概览</h2>
      <div class="overview-cards">
        <div class="overview-card" v-for="item in overviewItems" :key="item.id">
          <div class="card-icon">{{ item.icon }}</div>
          <div class="card-content">
            <div class="card-value">{{ item.value }}</div>
            <div class="card-label">{{ item.label }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 学习进度 -->
    <div class="progress-section">
      <h2 class="section-title">本周进度</h2>
      <div class="progress-card">
        <div class="progress-header">
          <span class="progress-label">学习完成度</span>
          <span class="progress-percentage">{{ weeklyProgress }}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: weeklyProgress + '%' }"></div>
        </div>
        <div class="progress-details">
          <span class="detail-item">{{ completedTasks }}/5 任务</span>
          <span class="detail-item">{{ studyTime }} 分钟学习</span>
        </div>
      </div>
    </div>

    <!-- 最近活动 -->
    <div class="activity-section">
      <div class="section-header">
        <h2 class="section-title">最近活动</h2>
        <button class="view-all" @click="$router.push('/manage')">查看全部</button>
      </div>
      <div class="activity-list">
        <div class="activity-item" v-for="activity in recentActivities" :key="(activity as any).id">
          <div class="activity-icon">{{ (activity as any).icon }}</div>
          <div class="activity-content">
            <div class="activity-title">{{ (activity as any).title }}</div>
            <div class="activity-time">{{ (activity as any).time }}</div>
          </div>
          <div class="activity-status" :class="(activity as any).status">{{ (activity as any).status }}</div>
        </div>
      </div>
    </div>

    <!-- 学习提示 -->
    <div class="tip-section">
      <div class="tip-card">
        <div class="tip-icon">💡</div>
        <div class="tip-content">
          <h3>学习小贴士</h3>
          <p>{{ dailyTip }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getQuestions } from '../apis/errorQuestions'
import { getSources } from '../apis/sources'
import { getSubjects } from '../apis/subjects'

// 概览数据
const overviewItems = ref([
  { id: 1, icon: '📚', value: 0, label: '总错题数' },
  { id: 2, icon: '✅', value: 0, label: '已掌握' },
  { id: 3, icon: '📅', value: 0, label: '今日待复习' },
  { id: 4, icon: '🔥', value: 0, label: '连续学习' }
])

// 进度数据
const weeklyProgress = ref(0)
const completedTasks = ref(0)
const studyTime = ref(0)

// 最近活动
const recentActivities = ref([])

// 学习提示
const dailyTip = ref('定期复习是巩固知识的最佳方法。研究表明，间隔重复学习比集中学习更有效。')
</script>

<style scoped>
.home-page.minimal {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  max-width: 800px;
  margin: 0 auto;
}

.minimal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 60px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.app-title {
  font-size: 28px;
  font-weight: 300;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: 1px;
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  overflow: hidden;
  transition: all 0.3s;
  border: 2px solid var(--primary-color);
  cursor: pointer;
  flex-shrink: 0;
}

.user-avatar:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.core-actions {
  margin-bottom: 60px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.primary-action {
  width: 100%;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 12px;
  padding: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  font-size: 18px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-sizing: border-box;
}

.primary-action:hover {
  background: var(--primary-dark);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.primary-action:active {
  transform: translateY(0);
}

.secondary-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  width: 100%;
}

.secondary-action {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  transition: all 0.3s;
  box-sizing: border-box;
}

.secondary-action:hover {
  background: var(--bg-secondary);
  border-color: var(--primary-color);
  transform: translateY(-2px);
}

.action-icon {
  font-size: 32px;
}

.action-text {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.primary-action .action-text {
  color: white;
}

.overview-section {
  margin-bottom: 60px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.section-title {
  font-size: 20px;
  font-weight: 400;
  color: var(--text-primary);
  margin: 0 0 30px 0;
  letter-spacing: 0.5px;
}

.overview-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  width: 100%;
}

.overview-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 32px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  transition: all 0.3s;
  border: 1px solid var(--border-color);
  box-sizing: border-box;
}

.overview-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.card-icon {
  font-size: 40px;
}

.card-content {
  text-align: center;
}

.card-value {
  font-size: 32px;
  font-weight: 300;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.card-label {
  font-size: 14px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.progress-section {
  margin-bottom: 60px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.progress-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 32px 24px;
  border: 1px solid var(--border-color);
  width: 100%;
  box-sizing: border-box;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.progress-label {
  font-size: 16px;
  color: var(--text-primary);
  font-weight: 400;
}

.progress-percentage {
  font-size: 18px;
  font-weight: 500;
  color: var(--primary-color);
}

.progress-track {
  width: 100%;
  height: 6px;
  background: var(--bg-secondary);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 20px;
}

.progress-fill {
  height: 100%;
  background: var(--primary-color);
  border-radius: 3px;
  transition: width 1s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-details {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
  color: var(--text-secondary);
}

.activity-section {
  margin-bottom: 60px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.view-all {
  background: none;
  border: none;
  color: var(--primary-color);
  font-size: 14px;
  cursor: pointer;
  padding: 0;
  transition: all 0.3s;
  flex-shrink: 0;
}

.view-all:hover {
  text-decoration: underline;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: 100%;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--card-bg);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  transition: all 0.3s;
  width: 100%;
  box-sizing: border-box;
}

.activity-item:hover {
  background: var(--bg-secondary);
  border-color: var(--primary-color);
}

.activity-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-size: 16px;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.activity-time {
  font-size: 14px;
  color: var(--text-secondary);
}

.activity-status {
  font-size: 12px;
  font-weight: 500;
  padding: 4px 12px;
  border-radius: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.activity-status.completed {
  background: rgba(76, 175, 80, 0.1);
  color: #4caf50;
}

.tip-section {
  margin-bottom: 60px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.tip-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 32px 24px;
  display: flex;
  align-items: flex-start;
  gap: 20px;
  border: 1px solid var(--border-color);
  border-left: 4px solid var(--primary-color);
  width: 100%;
  box-sizing: border-box;
}

.tip-icon {
  font-size: 28px;
  flex-shrink: 0;
  margin-top: 2px;
}

.tip-content h3 {
  font-size: 18px;
  font-weight: 400;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.tip-content p {
  font-size: 16px;
  line-height: 1.6;
  color: var(--text-secondary);
  margin: 0;
}

.ai-section {
  margin-bottom: 60px;
  width: 100%;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.ai-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 24px;
  border: 1px solid var(--border-color);
  border-left: 4px solid #9c27b0;
  width: 100%;
  box-sizing: border-box;
}

.ai-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.ai-icon {
  font-size: 24px;
}

.ai-header h3 {
  font-size: 18px;
  font-weight: 400;
  color: var(--text-primary);
  margin: 0;
}

.ai-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ai-status, .ai-token, .ai-model {
  display: flex;
  justify-content: space-between;
  align-items: center;
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
  .home-page.minimal {
    padding: 30px 20px;
  }
  
  .minimal-header {
    margin-bottom: 40px;
  }
  
  .core-actions, .overview-section, .progress-section, .activity-section, .tip-section, .ai-section {
    margin-bottom: 40px;
  }
}
</style>