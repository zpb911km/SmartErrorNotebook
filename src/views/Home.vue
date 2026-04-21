<template>
  <div class="home-page">
    <div class="header">
      <h1>智能错题本</h1>
      <p class="subtitle">让每一次错误，都成为进步的阶梯</p>
    </div>

    <div class="stats-cards">
      <div class="stat-card">
        <div class="stat-icon">📚</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.total }}</div>
          <div class="stat-label">总错题数</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">✅</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.mastered }}</div>
          <div class="stat-label">已掌握</div>
        </div>
      </div>
    </div>

    <div class="quick-actions">
      <h2>快捷操作</h2>
      <div class="action-grid">
        <button class="action-btn primary" @click="$router.push('/add')">
          <span class="btn-icon">➕</span>
          <span>添加错题</span>
        </button>
        <button class="action-btn" @click="$router.push('/review')">
          <span class="btn-icon">📖</span>
          <span>开始复习</span>
        </button>
        <button class="action-btn" @click="$router.push('/manage')">
          <span class="btn-icon">📋</span>
          <span>错题管理</span>
        </button>
        <button class="action-btn" @click="$router.push('/stats')">
          <span class="btn-icon">📊</span>
          <span>数据分析</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const stats = ref({
  total: 128,
  mastered: 89
})
</script>

<style scoped>
.home-page {
  padding: 20px;
  padding-bottom: 80px;
}

.header {
  text-align: center;
  margin-bottom: 30px;
}

.header h1 {
  font-size: 28px;
  color: var(--primary-color);
  margin: 0 0 8px 0;
}

.subtitle {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.stats-cards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 30px;
}

.stat-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  font-size: 32px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: var(--primary-color);
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.quick-actions {
  margin-bottom: 30px;
}

.quick-actions h2 {
  font-size: 18px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.action-btn {
  background: var(--card-bg);
  border: none;
  border-radius: 12px;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.3s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.action-btn.primary {
  background: var(--primary-color);
  color: white;
}

.action-btn:active {
  transform: scale(0.98);
}

.btn-icon {
  font-size: 32px;
}

.action-btn span:last-child {
  font-size: 14px;
  font-weight: 500;
}
</style>