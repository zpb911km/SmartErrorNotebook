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

      <!-- LLM 配置 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">⚙️</div>
          <div class="setting-name">LLM 配置</div>
        </div>
        <div class="setting-action">
          <button class="config-btn" @click="openLLMConfig">配置</button>
        </div>
      </div>

      <!-- LLM 测试 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">🧪</div>
          <div class="setting-name">LLM 测试</div>
        </div>
        <div class="setting-action">
          <button class="config-btn" @click="openLLMTest">测试</button>
        </div>
      </div>

        <!-- AI 提示词设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">✏️</div>
          <div class="setting-name">AI 提示词设置</div>
        </div>
        <div class="setting-action">
          <button class="config-btn" @click="showPromptEditor = true">编辑</button>
        </div>
      </div>

      <!--- Markdown 渲染测试-->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">📄</div>
          <div class="setting-name">Markdown 渲染测试</div>
        </div>
        <div class="setting-action">
          <button class="config-btn" @click="$router.push('/markdown-test')">测试</button>
        </div>
      </div>

      <!-- 数据清理 -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-icon">🗑️</div>
          <div class="setting-name">清理已同步的软删除数据</div>
        </div>
        <div class="setting-action">
          <button class="config-btn danger" @click="confirmPurge">清理</button>
        </div>
      </div>

      <!-- LLM 配置对话框 -->
      <div v-if="showLLMConfig" class="modal-overlay" @click="closeLLMConfig">
        <div class="modal" @click.stop>
          <div class="modal-header">
            <h3>LLM 配置</h3>
            <button class="close-btn" @click="closeLLMConfig">×</button>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label>Base URL</label>
              <input
                v-model="llmConfig.baseUrl"
                type="text"
                placeholder="https://api.openai.com"
                class="form-input"
              />
            </div>
            <div class="form-group">
              <label>API Key</label>
              <input
                v-model="llmConfig.apiKey"
                type="password"
                placeholder="sk-..."
                class="form-input"
              />
            </div>
            <div class="form-group">
              <label>Model</label>
              <input
                v-model="llmConfig.model"
                type="text"
                placeholder="gpt-3.5-turbo"
                class="form-input"
              />
            </div>
            <div class="form-group">
              <label>启用 LLM</label>
              <div class="toggle-switch">
                <input type="checkbox" v-model="llmConfig.enabled" id="llmEnabledToggle" />
                <label for="llmEnabledToggle" class="toggle-label"></label>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="closeLLMConfig">取消</button>
            <button class="btn btn-primary" @click="saveLLMConfig">保存</button>
          </div>
        </div>
      </div>


      <!-- LLM 测试对话框 -->
      <div v-if="showLLMTest" class="modal-overlay" @click="closeLLMTest">
        <div class="modal test-modal" @click.stop>
          <div class="modal-header">
            <h3>LLM 测试</h3>
            <button class="close-btn" @click="closeLLMTest">×</button>
          </div>
          <div class="modal-body test-body">
            <div class="test-status">
              <span :class="['status-dot', testStatus]"></span>
              <span class="status-text">{{ getStatusText() }}</span>
            </div>
            <div class="chat-container">
              <div class="chat-messages" ref="chatMessages">
                <div
                  v-for="(msg, index) in testMessages"
                  :key="index"
                  :class="['message', msg.role]"
                >
                  <div class="message-content">{{ msg.content }}</div>
                </div>
                <div v-if="isSending" class="message assistant">
                  <div class="message-content loading">
                    <span></span><span></span><span></span>
                  </div>
                </div>
              </div>
              <div class="chat-input">
                <input
                  v-model="testInput"
                  type="text"
                  placeholder="输入测试消息..."
                  @keypress.enter="sendTestMessage"
                  :disabled="isSending"
                />
                <button
                  @click="sendTestMessage"
                  :disabled="isSending || !testInput.trim()"
                  class="send-btn"
                >
                  发送
                </button>
              </div>
            </div>
            <button class="clear-chat-btn" @click="clearTestChat">清空对话</button>
          </div>
        </div>
      </div>

      <!-- 提示词编辑器对话框 -->
      <div v-if="showPromptEditor" class="modal-overlay" @click="closePromptEditor">
        <div class="modal large-modal" @click.stop>
          <div class="modal-header">
            <h3>AI 提示词设置</h3>
            <button class="close-btn" @click="closePromptEditor">×</button>
          </div>
          <div class="modal-body">
            <p class="prompt-description">
              自定义 AI 提取题目信息时使用的提示词。修改后点击保存即可生效。
            </p>
            <PromptEditor />
          </div>
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="closePromptEditor">关闭</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { llm } from '../services'
import PromptEditor from '../components/PromptEditor.vue'
import { purgeSyncedDeletions } from '../apis/sync'

// AI 选项
const aiEnabled = ref(false)
const aiAnalysis = ref(true)
const aiReview = ref(true)
const aiRecommend = ref(false)

// LLM 配置对话框
const showLLMConfig = ref(false)
const llmConfig = ref({
  baseUrl: '',
  apiKey: '',
  model: '',
  enabled: false
})

// LLM 测试对话框
const showLLMTest = ref(false)
const testStatus = ref<'idle' | 'success' | 'error'>('idle')
const testMessages = ref<Array<{ role: 'user' | 'assistant'; content: string }>>([])
const testInput = ref('')
const isSending = ref(false)
const chatMessages = ref<HTMLElement | null>(null)

// 提示词编辑器
const showPromptEditor = ref(false)

const closePromptEditor = () => {
  showPromptEditor.value = false
}

// 主题切换
const handleThemeChange = () => {
  console.log('主题切换为:', theme.value)
  applyTheme(theme.value)
  // 保存主题设置到localStorage
  localStorage.setItem('theme', theme.value)
}
const theme = ref(localStorage.getItem('theme') || 'system')
console.log('初始化 theme 值:', theme.value, 'localStorage 中的值:', localStorage.getItem('theme'))

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

// 系统主题变化处理
const handleSystemThemeChange = () => {
  console.log('系统主题变化事件触发，当前 theme.value:', theme.value, 'localStorage 中的值:', localStorage.getItem('theme'))
  if (theme.value === 'system') {
    console.log('执行跟随系统主题更新')
    applyTheme('system')
  } else {
    console.log('当前主题不是 system，不执行跟随系统主题更新，当前主题:', theme.value)
    // 确保应用当前设置的主题
    applyTheme(theme.value)
  }
}

// AI 选项切换
const handleAiToggle = () => {
  llm.updateConfig({ enabled: aiEnabled.value })
  console.log('AI', aiEnabled.value ? '开启' : '关闭')
}

// 打开 LLM 配置
const openLLMConfig = () => {
  // 加载当前配置
  llmConfig.value = {
    baseUrl: llm.config.baseUrl,
    apiKey: llm.config.apiKey,
    model: llm.config.model,
    enabled: llm.config.enabled
  }
  showLLMConfig.value = true
}

// 关闭 LLM 配置
const closeLLMConfig = () => {
  showLLMConfig.value = false
}

// 保存 LLM 配置
const saveLLMConfig = () => {
  llm.updateConfig({
    baseUrl: llmConfig.value.baseUrl,
    apiKey: llmConfig.value.apiKey,
    model: llmConfig.value.model,
    enabled: llmConfig.value.enabled
  })
  // 同步更新外部开关状态
  aiEnabled.value = llmConfig.value.enabled
  showLLMConfig.value = false
}

// 打开 LLM 测试
const openLLMTest = () => {
  if (!llm.config.enabled || !llm.isConfigured()) {
    alert('请先启用并配置 LLM')
    return
  }
  showLLMTest.value = true
  testStatus.value = 'success'
}

// 关闭 LLM 测试
const closeLLMTest = () => {
  showLLMTest.value = false
}

// 获取状态文本
const getStatusText = () => {
  switch (testStatus.value) {
    case 'idle':
      return '未测试'
    case 'success':
      return '连接正常'
    case 'error':
      return '连接失败'
  }
}

// 发送测试消息
const sendTestMessage = async () => {
  if (!testInput.value.trim() || isSending.value) return

  const userMessage = testInput.value.trim()
  testInput.value = ''

  // 添加用户消息
  testMessages.value.push({ role: 'user', content: userMessage })
  
  // 滚动到底部
  await scrollToBottom()

  isSending.value = true

  try {
    // 构建对话历史
    const history = testMessages.value
    
    const response = await llm.chat(history)
    
    testMessages.value.push({ role: 'assistant', content: response })
    testStatus.value = 'success'
  } catch (error) {
    console.error('LLM 测试失败:', error)
    testMessages.value.push({ 
      role: 'assistant', 
      content: `错误: ${error instanceof Error ? error.message : '未知错误'}` 
    })
    testStatus.value = 'error'
  } finally {
    isSending.value = false
    await scrollToBottom()
  }
}

// 滚动到底部
const scrollToBottom = async () => {
  await nextTick()
  if (chatMessages.value) {
    chatMessages.value.scrollTop = chatMessages.value.scrollHeight
  }
}

// 清空测试对话
const clearTestChat = () => {
  testMessages.value = []
  testStatus.value = 'idle'
}

// 确认并执行清理
const confirmPurge = async () => {
  const ok = confirm('确定要清理所有已同步且已软删除的记录吗？此操作不可恢复！')
  if (!ok) return

  try {
    const result = await purgeSyncedDeletions()
    const parts: string[] = []
    let total = 0
    for (const [table, info] of Object.entries(result)) {
      if (info.deleted > 0) {
        parts.push(`${table}: ${info.deleted} 条`)
        total += info.deleted
      }
    }
    if (total > 0) {
      alert(`清理完成！共删除 ${total} 条记录\n${parts.join('\n')}`)
    } else {
      alert('没有需要清理的记录')
    }
  } catch (e) {
    alert(`清理失败：${e}`)
  }
}

// 初始化主题
onMounted(() => {
  // 应用主题
  console.log('初始化主题:', theme.value)
  applyTheme(theme.value)
  
  // 监听系统主题变化
  if (window.matchMedia) {
    console.log('添加系统主题变化监听器')
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', handleSystemThemeChange)
  }
  
  // 加载 AI 配置
  aiEnabled.value = llm.config.enabled
})

// 组件卸载时移除监听器
onUnmounted(() => {
  if (window.matchMedia) {
    console.log('移除系统主题变化监听器')
    window.matchMedia('(prefers-color-scheme: dark)').removeEventListener('change', handleSystemThemeChange)
  }
})
</script>

<style scoped>
.settings-page {
  padding: 40px 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
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

/* 配置按钮 */
.config-btn {
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.config-btn:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.config-btn.danger {
  background: #ef4444;
}

.config-btn.danger:hover {
  background: #dc2626;
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

/* 模态框 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--card-bg);
  border-radius: 16px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.3s;
}

.close-btn:hover {
  background: var(--input-bg);
  color: var(--text-primary);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px;
  border-top: 1px solid var(--border-color);
}

/* 表单样式 */
.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 500;
}

.form-input {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.3s;
}

.form-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-input::placeholder {
  color: var(--text-secondary);
}

/* 按钮 */
.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-secondary {
  background: var(--input-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--border-color);
}

/* 测试对话框 */
.test-modal {
  max-width: 600px;
}

.test-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.test-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--input-bg);
  border-radius: 8px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--border-color);
}

.status-dot.success {
  background: #10b981;
}

.status-dot.error {
  background: #ef4444;
}

.status-text {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 聊天界面 */
.chat-container {
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  background: var(--input-bg);
}

.chat-messages {
  height: 300px;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message {
  display: flex;
  max-width: 80%;
}

.message.user {
  align-self: flex-end;
}

.message.assistant {
  align-self: flex-start;
}

.message-content {
  padding: 12px 16px;
  border-radius: 12px;
  font-size: 14px;
  line-height: 1.5;
  word-wrap: break-word;
}

.message.user .message-content {
  background: var(--primary-color);
  color: white;
  border-bottom-right-radius: 4px;
}

.message.assistant .message-content {
  background: var(--card-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-bottom-left-radius: 4px;
}

.message-content.loading {
  display: flex;
  gap: 4px;
  padding: 16px;
}

.message-content.loading span {
  width: 8px;
  height: 8px;
  background: var(--text-secondary);
  border-radius: 50%;
  animation: bounce 1.4s infinite ease-in-out both;
}

.message-content.loading span:nth-child(1) {
  animation-delay: -0.32s;
}

.message-content.loading span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

.chat-input {
  display: flex;
  gap: 8px;
  padding: 12px;
  border-top: 1px solid var(--border-color);
}

.chat-input input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.3s;
}

.chat-input input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.chat-input input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.chat-input input::placeholder {
  color: var(--text-secondary);
}

.send-btn {
  padding: 10px 20px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
  white-space: nowrap;
}

.send-btn:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.clear-chat-btn {
  padding: 8px 16px;
  background: var(--input-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
  align-self: flex-end;
}

.clear-chat-btn:hover {
  background: var(--border-color);
}

/* 提示词编辑器对话框 */
.large-modal {
  width: 95%;
  max-width: 800px;
  max-height: 90vh;
}

.prompt-description {
  margin-bottom: 20px;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
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

  .modal {
    width: 95%;
    max-height: 95vh;
  }

  .chat-messages {
    height: 250px;
  }
}
</style>
