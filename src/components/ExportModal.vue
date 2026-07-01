<template>
  <div class="export-modal-overlay" @click.self="handleClose">
    <div class="export-modal">
      <div class="modal-header">
        <h2 class="modal-title">导出错题</h2>
        <button class="modal-close-btn" @click="handleClose">
          <Icon name="x" :size="18" />
        </button>
      </div>

      <div class="modal-body">
        <div class="export-info">
          <Icon name="info" :size="16" />
          <span>当前筛选条件下共 <strong>{{ questions.length }}</strong> 道错题</span>
        </div>

        <div class="export-formats">
          <!-- JSON 导出 -->
          <div class="format-card" @click="handleExportJSON">
            <div class="format-icon json-icon">
              <Icon name="file-text" :size="28" />
            </div>
            <div class="format-info">
              <div class="format-name">导出为 JSON</div>
              <div class="format-desc">纯净数据格式，仅包含题目、答案和解析，适合备份与迁移</div>
            </div>
            <div class="format-action">
              <span class="action-btn">导出</span>
            </div>
          </div>

          <!-- HTML 导出（替代 PDF） -->
          <div class="format-card" @click="handleExportHTML">
            <div class="format-icon html-icon">
              <Icon name="file-text" :size="28" />
            </div>
            <div class="format-info">
              <div class="format-name">导出为 HTML</div>
              <div class="format-desc">
                仅含题目，方便打印练习，完整排版，公式精确。<br>
                用浏览器打开后按 Ctrl+P 即可保存为 PDF
              </div>
            </div>
            <div class="format-action">
              <span class="action-btn html-btn">导出</span>
            </div>
          </div>
        </div>

        <div class="export-note">
          <Icon name="info" :size="14" />
          <span>
            <strong>HTML 推荐方案</strong> — 浏览器原生渲染，数学公式排版与 App 内完全一致。
            导出后用 Chrome/Edge 打开，按 Ctrl+P 可选择「另存为 PDF」，效果远胜截图方案。
          </span>
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" @click="handleClose">取消</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Icon from './Icon.vue'
import type { ErrorQuestion } from '../types'
import { exportQuestionsToJSON } from '../utils/exportJson'
import { exportQuestionsToHTML } from '../utils/exportHtml'
import { showError } from '../utils/notification'

const props = defineProps<{
  questions: ErrorQuestion[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const isExporting = ref(false)

const handleClose = () => {
  if (!isExporting.value) emit('close')
}

const handleExportJSON = async () => {
  if (isExporting.value) return
  if (!props.questions.length) { showError('导出失败','没有可导出的错题'); return }
  isExporting.value = true
  try { await exportQuestionsToJSON(props.questions) }
  finally { isExporting.value = false; emit('close') }
}

const handleExportHTML = async () => {
  if (isExporting.value) return
  if (!props.questions.length) { showError('导出失败','没有可导出的错题'); return }
  isExporting.value = true
  try { await exportQuestionsToHTML(props.questions) }
  finally { isExporting.value = false; emit('close') }
}
</script>

<style scoped>
.export-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: fadeIn 0.2s ease-out;
}

.export-modal {
  background: var(--card-bg);
  border-radius: var(--radius-lg);
  width: 480px;
  max-width: 90vw;
  max-height: 80vh;
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  animation: slideUp 0.25s ease-out;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.modal-close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.modal-close-btn:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.export-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--info-light);
  border-radius: var(--radius-md);
  color: var(--info-color);
  font-size: 14px;
  margin-bottom: 20px;
}

.export-info strong { font-weight: 700; }

.export-formats {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.format-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
}

.format-card:hover {
  border-color: var(--primary-color);
  background: var(--primary-light);
}

.format-icon {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.json-icon { color: var(--primary-color); }
.html-icon { color: var(--success-color); }

.format-info { flex: 1; }

.format-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.format-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.format-action { flex-shrink: 0; }

.action-btn {
  display: inline-block;
  padding: 6px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: var(--radius-sm);
  background: var(--primary-color);
  color: white;
}

.html-btn {
  background: var(--success-color);
}

.export-note {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 14px;
  background: var(--warning-light);
  border-radius: var(--radius-md);
  color: var(--warning-color);
  font-size: 12px;
  line-height: 1.5;
}

.export-note > :first-child {
  flex-shrink: 0;
  margin-top: 2px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
}

.cancel-btn {
  padding: 8px 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-btn:hover {
  background: var(--bg-tertiary);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
