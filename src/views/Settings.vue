<template>
  <div class="settings-page">
    <q-card flat bordered class="settings-section">
      <h1>设置</h1>
      <p class="text-muted">按你的习惯，调整学习体验。</p>
      <h2 class="settings-group-title">外观</h2>

      <!-- 主题设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <Icon name="palette" :size="22" class="setting-icon" />
          <div class="setting-name">主题设置</div>
        </div>
        <div class="setting-action">
          <q-select
            v-model="theme"
            outlined
            dense
            emit-value
            map-options
            :options="[
              { label: '浅色主题', value: 'light' },
              { label: '深色主题', value: 'dark' },
              { label: '跟随系统', value: 'system' }
            ]"
            aria-label="主题设置"
            class="theme-select"
            @update:model-value="handleThemeChange"
          />
        </div>
      </div>

      <h2 class="settings-group-title">AI 辅助与模型</h2>
      <!-- AI 选项 -->
      <div class="setting-item">
        <div class="setting-info">
          <Icon name="bot" :size="22" class="setting-icon" />
          <div class="setting-name">AI 辅助</div>
        </div>
        <div class="setting-action">
          <div class="toggle-switch">
            <q-toggle
              id="aiToggle"
              v-model="aiEnabled"
              color="primary"
              aria-label="AI 辅助"
              @update:model-value="handleAiToggle"
            />
          </div>
        </div>
      </div>

      <!-- LLM 配置 -->
      <div class="setting-item">
        <div class="setting-info">
          <Icon name="settings" :size="22" class="setting-icon" />
          <div class="setting-name">LLM 配置</div>
        </div>
        <div class="setting-action">
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="config-btn"
            @click="openLLMConfig"
          >
            配置
          </q-btn>
          <q-btn flat label="测试" @click="openLLMTest" />
        </div>
      </div>

      <!-- AI 提示词设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <Icon name="pencil" :size="22" class="setting-icon" />
          <div class="setting-name">AI 提示词设置</div>
        </div>
        <div class="setting-action">
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="config-btn"
            @click="showPromptEditor = true"
          >
            编辑
          </q-btn>
        </div>
      </div>

      <h2 class="settings-group-title">导出偏好</h2>
      <!-- 导出设置 -->
      <div class="setting-item">
        <div class="setting-info">
          <Icon name="file-text" :size="22" class="setting-icon" />
          <div class="setting-name">HTML 导出包含答案和解析</div>
        </div>
        <div class="setting-action">
          <div class="toggle-switch">
            <q-toggle
              id="exportAnswerToggle"
              v-model="exportIncludeAnswer"
              color="primary"
              aria-label="HTML 导出包含答案和解析"
              @update:model-value="handleExportAnswerToggle"
            />
          </div>
        </div>
      </div>

      <h2 class="settings-group-title">数据维护</h2>
      <!-- 数据清理 -->
      <div class="setting-item">
        <div class="setting-info">
          <Icon name="trash-2" :size="22" class="setting-icon" />
          <div class="setting-name">清理已同步的软删除数据</div>
        </div>
        <div class="setting-action">
          <q-btn
            no-caps
            unelevated
            type="button"
            color="negative"
            class="config-btn danger"
            :loading="purging"
            @click="confirmPurge"
          >
            清理
          </q-btn>
        </div>
      </div>

      <!-- LLM 配置对话框 -->
      <q-dialog
        class="notebook-dialog"
        :model-value="showLLMConfig"
        @hide="closeLLMConfig"
      >
        <div class="modal" @click.stop>
          <div class="modal-header">
            <h3>LLM 配置</h3>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="close-btn"
              @click="closeLLMConfig"
            >
              <Icon name="x" :size="18" />
            </q-btn>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label>Base URL</label>
              <q-input
                v-model="llmConfig.baseUrl"
                outlined
                dense
                aria-label="https://api.openai.com"
                type="text"
                placeholder="https://api.openai.com"
                class="form-input"
              />
            </div>
            <div class="form-group">
              <label>API Key</label>
              <q-input
                v-model="llmConfig.apiKey"
                outlined
                dense
                aria-label="sk-..."
                type="password"
                placeholder="sk-..."
                class="form-input"
              />
            </div>
            <div class="form-group">
              <label>Model</label>
              <q-input
                v-model="llmConfig.model"
                outlined
                dense
                aria-label="gpt-3.5-turbo"
                type="text"
                placeholder="gpt-3.5-turbo"
                class="form-input"
              />
            </div>
            <div class="form-group">
              <label>启用 LLM</label>
              <div class="toggle-switch">
                <q-toggle
                  id="llmEnabledToggle"
                  v-model="llmConfig.enabled"
                  color="primary"
                  aria-label="启用 LLM"
                />
                <label for="llmEnabledToggle" class="toggle-label" />
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="btn btn-secondary"
              @click="closeLLMConfig"
            >
              取消
            </q-btn>
            <q-btn
              no-caps
              unelevated
              type="button"
              color="primary"
              class="btn btn-primary"
              @click="saveLLMConfig"
            >
              保存
            </q-btn>
          </div>
        </div>
      </q-dialog>

      <!-- LLM 测试对话框 -->
      <q-dialog
        class="notebook-dialog"
        :model-value="showLLMTest"
        @hide="closeLLMTest"
      >
        <div class="modal test-modal" @click.stop>
          <div class="modal-header">
            <h3>LLM 测试</h3>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="close-btn"
              @click="closeLLMTest"
            >
              <Icon name="x" :size="18" />
            </q-btn>
          </div>
          <div class="modal-body test-body">
            <div class="test-status">
              <span :class="['status-dot', testStatus]" />
              <span class="status-text">{{ getStatusText() }}</span>
            </div>
            <div class="chat-container">
              <div ref="chatMessages" class="chat-messages">
                <div
                  v-for="(msg, index) in testMessages"
                  :key="index"
                  :class="['message', msg.role]"
                >
                  <div class="message-content">
                    {{ msg.content }}
                  </div>
                </div>
                <div v-if="isSending" class="message assistant">
                  <div class="message-content loading">
                    <span /><span /><span />
                  </div>
                </div>
              </div>
              <div class="chat-input">
                <q-input
                  v-model="testInput"
                  outlined
                  dense
                  aria-label="输入测试消息..."
                  type="text"
                  placeholder="输入测试消息..."
                  :disable="isSending"
                  @keypress.enter="sendTestMessage"
                />
                <q-btn
                  no-caps
                  unelevated
                  type="button"
                  flat
                  :disable="isSending || !testInput.trim()"
                  class="send-btn"
                  @click="sendTestMessage"
                >
                  发送
                </q-btn>
              </div>
            </div>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="clear-chat-btn"
              @click="clearTestChat"
            >
              清空对话
            </q-btn>
          </div>
        </div>
      </q-dialog>

      <!-- 提示词编辑器对话框 -->
      <q-dialog
        class="notebook-dialog"
        :model-value="showPromptEditor"
        @hide="closePromptEditor"
      >
        <div class="modal large-modal" @click.stop>
          <div class="modal-header">
            <h3>AI 提示词设置</h3>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="close-btn"
              @click="closePromptEditor"
            >
              <Icon name="x" :size="18" />
            </q-btn>
          </div>
          <div class="modal-body">
            <p class="prompt-description">
              自定义 AI 提取题目信息时使用的提示词。修改后点击保存即可生效。
            </p>
            <PromptEditor />
          </div>
          <div class="modal-footer">
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="btn btn-secondary"
              @click="closePromptEditor"
            >
              关闭
            </q-btn>
          </div>
        </div>
      </q-dialog>
    </q-card>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'

import {
  checkAndDeleteOrphans,
  purgeSyncedDeletions
} from '../api/platformExceptions'
import PromptEditor from '../components/PromptEditor.vue'
import { useLatestRequest } from '../composables/useLatestRequest'
import { setTheme, theme } from '../composables/useTheme'
import { llm } from '../services'
import { confirmAction, showAlert } from '../utils/dialog'
import { showInfo, showSuccess } from '../utils/notification'

// AI 选项
const aiEnabled = ref(false)

// LLM 配置对话框
const showLLMConfig = ref(false)

// 导出设置
const exportIncludeAnswer = ref(
  localStorage.getItem('export_include_answer') === 'true'
)
const handleExportAnswerToggle = () => {
  localStorage.setItem(
    'export_include_answer',
    String(exportIncludeAnswer.value)
  )
}
const llmConfig = ref({
  baseUrl: '',
  apiKey: '',
  model: '',
  enabled: false
})

// LLM 测试对话框
const showLLMTest = ref(false)
const beginTestRequest = useLatestRequest()
const testStatus = ref<'idle' | 'success' | 'error'>('idle')
const testMessages = ref<
  Array<{ role: 'user' | 'assistant'; content: string }>
>([])
const testInput = ref('')
const isSending = ref(false)
const chatMessages = ref<HTMLElement | null>(null)

// 提示词编辑器
const showPromptEditor = ref(false)

const closePromptEditor = () => {
  showPromptEditor.value = false
}

const handleThemeChange = () => setTheme(theme.value)

// AI 选项切换
const handleAiToggle = () => {
  llm.updateConfig({ enabled: aiEnabled.value })
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
    showAlert('请先启用并配置 LLM')
    return
  }
  showLLMTest.value = true
  testStatus.value = 'idle'
}

// 关闭 LLM 测试
const closeLLMTest = () => {
  beginTestRequest()
  isSending.value = false
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
  const isCurrent = beginTestRequest()
  isSending.value = true

  const userMessage = testInput.value.trim()
  testInput.value = ''

  // 添加用户消息
  testMessages.value.push({ role: 'user', content: userMessage })

  // 滚动到底部
  await scrollToBottom()

  try {
    // 构建对话历史
    const history = testMessages.value.map((message) => ({ ...message }))

    const response = await llm.chat(history)
    if (!isCurrent()) return

    testMessages.value.push({ role: 'assistant', content: response })
    testStatus.value = 'success'
  } catch (error) {
    if (!isCurrent()) return
    console.error('LLM 测试失败:', error)
    testMessages.value.push({
      role: 'assistant',
      content: `错误: ${error instanceof Error ? error.message : '未知错误'}`
    })
    testStatus.value = 'error'
  } finally {
    if (isCurrent()) {
      isSending.value = false
      await scrollToBottom()
    }
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
  beginTestRequest()
  isSending.value = false
  testMessages.value = []
  testStatus.value = 'idle'
}

// 确认并执行清理
const purging = ref(false)
const confirmPurge = async () => {
  if (purging.value) return
  purging.value = true
  const ok = await confirmAction(
    '确定要清理所有已同步且已软删除的记录吗？此操作不可恢复！'
  )
  if (!ok) {
    purging.value = false
    return
  }

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
      showSuccess(
        `清理完成！`,
        `共删除 ${total} 条记录\n${parts.join('\n')}`,
        5000
      )
    } else {
      showInfo('没有需要清理的记录', '')
    }
    const orphans = await checkAndDeleteOrphans()
    showSuccess(
      '自动检查完成',
      `共检查了${orphans.total_checked}条记录\n已删除${orphans.orphan_records_soft_deleted.length}条无效记录`,
      5000
    )
  } catch (e) {
    await showAlert(`清理失败：${e}`)
  } finally {
    purging.value = false
  }
}

// 初始化 AI 开关；主题由应用壳统一管理。
onMounted(() => {
  aiEnabled.value = llm.config.enabled
})
</script>
