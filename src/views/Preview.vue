<template>
  <div class="preview-page">
    <!-- 最近错题部分 (来自 Home) -->
    <div class="recent-errors">
      <h2>最近错题</h2>
      <div class="error-list">
        <div v-for="error in recentErrors" :key="error.id" class="error-item">
          <div class="error-subject">{{ error.subject }}</div>
          <div class="error-content">{{ error.content }}</div>
          <div class="error-meta">
            <span class="error-date">{{ error.date }}</span>
            <span class="error-status" :class="error.status">{{ error.statusText }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 薄弱环节部分 (来自 Stats) -->
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
import { useRouter } from 'vue-router'

const router = useRouter()

// 最近错题数据 (来自 Home)
const recentErrors = ref([
  {
    id: 1,
    subject: '数学',
    content: '求函数 f(x) = x² - 2x + 1 的最小值',
    date: '2026-01-15',
    status: 'pending',
    statusText: '待复习'
  },
  {
    id: 2,
    subject: '物理',
    content: '一个质量为 2kg 的物体从 10m 高处自由落下',
    date: '2026-01-14',
    status: 'reviewed',
    statusText: '已复习'
  },
  {
    id: 3,
    subject: '英语',
    content: '翻译句子：The quick brown fox jumps over the lazy dog',
    date: '2026-01-13',
    status: 'mastered',
    statusText: '已掌握'
  }
])

// 薄弱环节数据 (来自 Stats)
const weakPoints = ref([
  { name: '函数与导数', count: 15 },
  { name: '牛顿运动定律', count: 12 },
  { name: '化学方程式配平', count: 10 },
  { name: '英语语法', count: 8 }
])

// 点击复习按钮，跳转到 Review-Detail 组件
const reviewWeak = (item: any) => {
  router.push({
    path: '/review-detail',
    query: { topic: item.name }
  })
}
</script>

<style scoped>
.preview-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  margin: 0 auto;
}

/* 最近错题样式 (来自 Home) */
.recent-errors h2 {
  font-size: 18px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.error-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.error-item {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.error-subject {
  display: inline-block;
  padding: 4px 8px;
  background: var(--primary-light);
  color: var(--primary-color);
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  margin-bottom: 8px;
}

.error-content {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 12px;
  line-height: 1.5;
}

.error-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.error-date {
  color: var(--text-secondary);
}

.error-status {
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.error-status.pending {
  background: #fff3e0;
  color: #e65100;
}

.error-status.reviewed {
  background: #e3f2fd;
  color: #1976d2;
}

.error-status.mastered {
  background: #e8f5e9;
  color: #43a047;
}

/* 薄弱环节样式 (来自 Stats) */
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
