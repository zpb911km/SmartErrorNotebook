<template>
  <div class="settings-page">
    <div class="settings-section">
      <h3>设置选项</h3>
      
      <!-- 主题设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">🎨</div>
          <div class="setting-name">主题设置</div>
        </div>
        <div class="setting-action">
          <select v-model="theme" class="theme-select" @change="handleThemeChange">
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
            <input type="checkbox" v-model="aiEnabled" id="aiToggle" @change="handleAiToggle" />
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
import { ref, onMounted } from 'vue'

// 主题设置
const theme = ref('light')

// AI 选项
const aiEnabled = ref(true)
const aiAnalysis = ref(true)
const aiReview = ref(true)
const aiRecommend = ref(false)

// 主题切换
const handleThemeChange = () => {
  console.log('主题切换为:', theme.value)
  applyTheme(theme.value)
  // 保存主题设置到localStorage
  localStorage.setItem('theme', theme.value)
}

// 应用主题
const applyTheme = (themeValue: string) => {
  // 移除所有主题类
  document.body.classList.remove('light-theme', 'dark-theme')
  
  if (themeValue === 'light') {
    document.body.classList.add('light-theme')
  } else if (themeValue === 'dark') {
    document.body.classList.add('dark-theme')
  } else if (themeValue === 'system') {
    // 跟随系统
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.body.classList.add('dark-theme')
    } else {
      document.body.classList.add('light-theme')
    }
  }
}

// 初始化主题
onMounted(() => {
  // 从localStorage读取主题设置
  const savedTheme = localStorage.getItem('theme')
  if (savedTheme) {
    theme.value = savedTheme
  } else {
    // 默认跟随系统
    theme.value = 'system'
  }
  // 应用主题
  applyTheme(theme.value)
  
  // 监听系统主题变化
  if (window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (theme.value === 'system') {
        applyTheme('system')
      }
    })
  }
})

// AI 选项切换
const handleAiToggle = () => {
  console.log('AI', aiEnabled.value ? '开启' : '关闭')
}
</script>

<style scoped>
.settings-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  max-width: 800px;
  margin: 0 auto;
}

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
