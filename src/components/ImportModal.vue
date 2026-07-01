<template>
  <div class="import-modal-overlay" @click.self="handleClose">
    <div class="import-modal">
      <!-- 标题 -->
      <div class="modal-header">
        <h2 class="modal-title">导入错题</h2>
        <button class="modal-close-btn" @click="handleClose">
          <Icon name="x" :size="18" />
        </button>
      </div>

      <div class="modal-body">
        <!-- 步骤 1：选择文件 -->
        <div v-if="step === 'select'" class="step-container">
          <div class="step-content">
            <div class="file-select-area" @click="handleSelectFile">
              <Icon name="file-text" :size="48" />
              <div class="file-select-text">
                <div class="file-select-title">点击选择 JSON 文件</div>
                <div class="file-select-desc">
                  支持符合 SmartErrorNotebook 导出格式的 JSON 文件
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 步骤 2：配置导入 -->
        <div v-else-if="step === 'configure'" class="step-container">
          <div class="step-header">
            <div class="step-title">导入预览</div>
            <div class="step-subtitle">
              共 {{ importData?.questions?.length || 0 }} 道题目待导入
            </div>
          </div>

          <!-- 文件信息 -->
          <div class="file-info-card">
            <div class="file-info-row">
              <span class="info-label">文件</span>
              <span class="info-value file-path" :title="fileName">{{ fileName }}</span>
            </div>
            <div class="file-info-row">
              <span class="info-label">版本</span>
              <span class="info-value">{{ importData?.version }}</span>
            </div>
            <div class="file-info-row">
              <span class="info-label">题目数</span>
              <span class="info-value">{{ importData?.questions?.length || 0 }}</span>
            </div>
          </div>

          <!-- 配置表单 -->
          <div class="config-form">
            <div class="form-group">
              <label class="form-label">目标科目 <span class="required">*</span></label>
              <SubjectSelector
                v-model="selectedSubjectId"
                @select="handleSubjectSelect"
              />
            </div>

            <div class="form-group">
              <label class="form-label">默认题型 <span class="required">*</span></label>
              <div class="type-select-wrapper">
                <select v-model="selectedType" class="type-select">
                  <option
                    v-for="type in questionTypes"
                    :key="type.value"
                    :value="type.value"
                  >
                    {{ type.label }}
                  </option>
                </select>
              </div>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="step-actions">
            <button class="back-btn" @click="step = 'select'">重新选择</button>
            <button
              class="import-btn"
              :disabled="!canImport || isImporting"
              @click="handleImport"
            >
              {{ isImporting ? '导入中...' : `导入 ${importData?.questions?.length || 0} 题` }}
            </button>
          </div>
        </div>

        <!-- 错误状态 -->
        <div v-if="errorMsg" class="error-card">
          <div class="error-icon">
            <Icon name="circle-x" :size="24" />
          </div>
          <div class="error-text">{{ errorMsg }}</div>
          <button class="retry-btn" @click="resetState">重试</button>
        </div>
      </div>

      <!-- 底部 -->
      <div class="modal-footer">
        <button class="cancel-btn" @click="handleClose">关闭</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import Icon from './Icon.vue'
import SubjectSelector from './SubjectSelector.vue'
import type { ExportJSONSchema } from '../types'
import {
  readImportFile,
  importQuestions,
  getCurrentUserId
} from '../utils/importJson'
import { showError } from '../utils/notification'
import { QuestionType } from '../types'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'import-complete'): void
}>()

// 步骤状态
const step = ref<'select' | 'configure'>('select')
const errorMsg = ref('')
const isImporting = ref(false)

// 文件数据
const fileName = ref('')
const importData = ref<ExportJSONSchema | null>(null)

// 配置
const selectedSubjectId = ref('')
const selectedType = ref(QuestionType.ShortAnswer)
const questionTypes = computed(() => {
  return Object.entries(QuestionType).map(([value, label]) => ({
    value,
    label
  }))
})

const canImport = computed(() => {
  return selectedSubjectId.value && selectedType.value && importData.value
})

const handleClose = () => {
  if (!isImporting.value) {
    emit('close')
  }
}

const resetState = () => {
  step.value = 'select'
  errorMsg.value = ''
  fileName.value = ''
  importData.value = null
  selectedSubjectId.value = ''
  selectedType.value = QuestionType.ShortAnswer
}

const handleSubjectSelect = (subjectId: string) => {
  selectedSubjectId.value = subjectId
}

// 选择文件
const handleSelectFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const filePath = await open({
      multiple: false,
      filters: [
        {
          name: 'JSON 文件',
          extensions: ['json']
        }
      ]
    })

    if (!filePath) return

    // 解析文件
    const data = await readImportFile(filePath as string)
    fileName.value = (filePath as string).split('\\').pop()?.split('/').pop() || ''
    importData.value = data
    step.value = 'configure'
    errorMsg.value = ''
  } catch (error) {
    errorMsg.value = String(error)
    console.error('选择文件失败:', error)
  }
}

// 执行导入
const handleImport = async () => {
  if (!canImport.value || !importData.value) return
  if (!selectedSubjectId.value) {
    showError('导入失败', '请选择目标科目')
    return
  }

  isImporting.value = true
  try {
    const userId = getCurrentUserId()
    await importQuestions(
      importData.value,
      selectedSubjectId.value,
      selectedType.value,
      userId
    )
    emit('import-complete')
    emit('close')
  } catch (error) {
    errorMsg.value = `导入失败: ${String(error)}`
    showError('导入失败', String(error))
  } finally {
    isImporting.value = false
  }
}
</script>

<style scoped>
.import-modal-overlay {
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

.import-modal {
  background: var(--card-bg);
  border-radius: var(--radius-lg);
  width: 500px;
  max-width: 90vw;
  max-height: 85vh;
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

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
  gap: 8px;
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

/* 步骤容器 */
.step-container {
  animation: fadeIn 0.2s ease-out;
}

.step-header {
  margin-bottom: 16px;
}

.step-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.step-subtitle {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 文件选择区 */
.file-select-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px 20px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-secondary);
}

.file-select-area:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
  background: var(--primary-light);
}

.file-select-text {
  text-align: center;
}

.file-select-title {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 4px;
}

.file-select-desc {
  font-size: 13px;
  color: var(--text-hint);
}

/* 文件信息卡片 */
.file-info-card {
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  margin-bottom: 16px;
}

.file-info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  font-size: 14px;
}

.file-info-row + .file-info-row {
  border-top: 1px solid var(--border-color);
}

.info-label {
  color: var(--text-secondary);
}

.info-value {
  color: var(--text-primary);
  font-weight: 500;
}

.file-path {
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 配置表单 */
.config-form {
  margin-bottom: 16px;
}

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.required {
  color: var(--danger-color);
}

.type-select-wrapper {
  position: relative;
}

.type-select {
  width: 100%;
  padding: 10px 12px;
  font-size: 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--input-bg);
  color: var(--text-primary);
  appearance: auto;
  cursor: pointer;
  transition: border-color 0.2s;
}

.type-select:focus {
  outline: none;
  border-color: var(--primary-color);
}

/* 步骤操作 */
.step-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.back-btn {
  padding: 8px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover {
  background: var(--bg-tertiary);
}

.import-btn {
  padding: 8px 20px;
  background: var(--primary-color);
  border: none;
  border-radius: var(--radius-md);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.import-btn:hover {
  background: var(--primary-dark);
}

.import-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 错误状态 */
.error-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 30px 20px;
}

.error-icon {
  color: var(--danger-color);
}

.error-text {
  font-size: 14px;
  color: var(--danger-color);
  text-align: center;
  line-height: 1.5;
  white-space: pre-wrap;
}

.retry-btn {
  padding: 8px 20px;
  background: var(--primary-color);
  border: none;
  border-radius: var(--radius-md);
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.retry-btn:hover {
  background: var(--primary-dark);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
