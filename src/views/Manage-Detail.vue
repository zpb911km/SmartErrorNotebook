<template>
  <div class="manage-detail-page">
    <!-- 返回按钮 -->
    <div class="header-section">
      <button class="back-button" @click="goBack">
        ← 返回错题管理
      </button>
      <h1>错题详情</h1>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 错题详情内容 -->
    <div v-else-if="errorQuestion" class="detail-content">
      <!-- 基本信息卡片 -->
      <div class="info-card">
        <div class="card-header">
          <span 
            class="subject-tag" 
            :style="getSubjectStyle(errorQuestion.subject_id)"
          >
            {{ getSubjectName(errorQuestion.subject_id) }}
          </span>
          <span class="question-type">{{ getQuestionTypeName(errorQuestion.type) }}</span>
        </div>

        <div class="source-info">
          <div class="source-item">
            <span class="label">来源：</span>
            <span class="value">{{ getSourceInfo()?.book || '未知' }}</span>
          </div>
          <div class="source-item">
            <span class="label">章节：</span>
            <span class="value">{{ getSourceInfo()?.chapter || '未知' }}</span>
          </div>
          <div class="source-item">
            <span class="label">知识点：</span>
            <span class="value">{{ getSourceInfo()?.knowledge || '未知' }}</span>
          </div>
        </div>
      </div>

      <!-- 题目内容 -->
      <div class="question-card">
        <h3>题目内容</h3>
        <div class="question-prompt">{{ errorQuestion.prompt }}</div>
        
        <!-- 附件 -->
        <div v-if="attachments.length > 0" class="attachments-section">
          <h4>相关图片</h4>
          <div class="attachments-grid">
            <div 
              v-for="attachment in attachments" 
              :key="attachment.id"
              class="attachment-item"
            >
              <img :src="attachment.base64" :alt="attachment.filename" />
            </div>
          </div>
        </div>
      </div>

      <!-- 答案部分 -->
      <div class="answer-card">
        <div class="answer-header">
          <h3>参考答案</h3>
          <button 
            v-if="!showAnswer" 
            class="view-answer-btn"
            @click="showAnswer = true"
          >
            查看答案
          </button>
        </div>
        
        <div v-if="showAnswer" class="answer-content">
          <div class="answer-text">{{ errorQuestion.answer }}</div>
          
          <div v-if="errorQuestion.analysis" class="analysis-section">
            <h4>解析</h4>
            <p>{{ errorQuestion.analysis }}</p>
          </div>
          
          <div v-if="errorQuestion.error_note" class="error-note-section">
            <h4>错误笔记</h4>
            <p>{{ errorQuestion.error_note }}</p>
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="action-buttons">
        <button class="btn edit-btn" @click="openEditModal">
          编辑信息
        </button>
        <button class="btn delete-btn" @click="confirmDelete">
          删除错题
        </button>
      </div>
    </div>

    <!-- 编辑模态框 -->
    <div v-if="showEditModal" class="modal-overlay" @click="closeEditModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>编辑错题信息</h2>
          <button class="close-btn" @click="closeEditModal">×</button>
        </div>
        
        <div class="modal-body">
          <div class="form-group">
            <label>题目内容</label>
            <textarea 
              v-model="editForm.prompt" 
              rows="3"
              placeholder="请输入题目内容"
            ></textarea>
          </div>
          
          <div class="form-group">
            <label>参考答案</label>
            <textarea 
              v-model="editForm.answer" 
              rows="3"
              placeholder="请输入参考答案"
            ></textarea>
          </div>
          
          <div class="form-group">
            <label>题目解析</label>
            <textarea 
              v-model="editForm.analysis" 
              rows="3"
              placeholder="请输入题目解析"
            ></textarea>
          </div>
          
          <div class="form-group">
            <label>错误笔记</label>
            <textarea 
              v-model="editForm.error_note" 
              rows="2"
              placeholder="请输入错误笔记"
            ></textarea>
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="btn cancel-btn" @click="closeEditModal">取消</button>
          <button class="btn save-btn" @click="saveChanges" :disabled="saving">
            {{ saving ? '保存中...' : '保存' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <div v-if="showDeleteConfirm" class="confirm-overlay" @click="cancelDelete">
      <div class="confirm-dialog" @click.stop>
        <div class="confirm-header">
          <h3>确认删除</h3>
        </div>
        <div class="confirm-body">
          <p>确定要删除这道错题吗？此操作不可撤销。</p>
        </div>
        <div class="confirm-footer">
          <button class="btn cancel-btn" @click="cancelDelete">取消</button>
          <button class="btn danger-btn" @click="performDelete" :disabled="deleting">
            {{ deleting ? '删除中...' : '确认删除' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getDataUtils } from '@/types/testdata'
import { 
  getErrorQuestionDetailMock, 
  updateErrorQuestionMock, 
  deleteErrorQuestionMock 
} from '@/apis/errorQuestions'
import type { ErrorQuestion } from '@/types'

const route = useRoute()
const router = useRouter()

// 状态管理
const loading = ref(true)
const errorQuestion = ref<ErrorQuestion | null>(null)
const showAnswer = ref(false)
const attachments = ref<any[]>([])

// 编辑相关
const showEditModal = ref(false)
const saving = ref(false)
const editForm = ref({
  prompt: '',
  answer: '',
  analysis: '',
  error_note: ''
})

// 删除相关
const showDeleteConfirm = ref(false)
const deleting = ref(false)

// 获取错题详情
const loadErrorQuestion = async () => {
  try {
    const questionId = route.params.id as string
    if (!questionId) {
      throw new Error('缺少错题ID')
    }
    
    loading.value = true
    errorQuestion.value = await getErrorQuestionDetailMock(questionId)
    
    // 初始化编辑表单
    if (errorQuestion.value) {
      editForm.value = {
        prompt: errorQuestion.value.prompt,
        answer: errorQuestion.value.answer || '', // 处理可能为undefined的情况
        analysis: errorQuestion.value.analysis || '',
        error_note: errorQuestion.value.error_note || ''
      }
    }
    
    // 获取附件信息（如果有）
    // attachments.value = await getAttachments(questionId)
    
  } catch (error) {
    console.error('加载错题详情失败:', error)
    // 可以添加错误提示
  } finally {
    loading.value = false
  }
}

// 获取科目名称
const getSubjectName = (subjectId: string): string => {
  const subject = getDataUtils.getSubjectById(subjectId)
  return subject ? subject.name : '未知科目'
}

// 获取科目样式
const getSubjectStyle = (subjectId: string) => {
  const subject = getDataUtils.getSubjectById(subjectId)
  if (!subject || !subject.color) return {}
  
  const bgColor = subject.color
  const r = parseInt(bgColor.slice(1, 3), 16)
  const g = parseInt(bgColor.slice(3, 5), 16)
  const b = parseInt(bgColor.slice(5, 7), 16)
  const brightness = (r * 299 + g * 587 + b * 114) / 1000
  const textColor = brightness > 128 ? '#333333' : '#ffffff'
  
  return {
    backgroundColor: bgColor,
    color: textColor,
    border: 'none'
  }
}

// 获取题目类型名称
const getQuestionTypeName = (type: string): string => {
  const typeMap: Record<string, string> = {
    '计算题': '计算题',
    '翻译题': '翻译题',
    '简答题': '简答题',
    '选择题': '选择题',
    '填空题': '填空题'
  }
  return typeMap[type] || type
}

// 获取来源信息
const getSourceInfo = () => {
  if (!errorQuestion.value) return null
  return getDataUtils.getSourceByQuestionId(errorQuestion.value.id)
}

// 返回上一页
const goBack = () => {
  router.back()
}

// 打开编辑模态框
const openEditModal = () => {
  showEditModal.value = true
}

// 关闭编辑模态框
const closeEditModal = () => {
  showEditModal.value = false
}

// 保存更改
const saveChanges = async () => {
  if (!errorQuestion.value) return
  
  try {
    saving.value = true
    
    const updates = {
      prompt: editForm.value.prompt,
      answer: editForm.value.answer,
      analysis: editForm.value.analysis,
      error_note: editForm.value.error_note
    }
    
    const success = await updateErrorQuestionMock(errorQuestion.value.id, updates)
    
    if (success) {
      // 更新本地数据
      Object.assign(errorQuestion.value, updates)
      closeEditModal()
      // 可以添加成功提示
    }
  } catch (error) {
    console.error('保存失败:', error)
    // 可以添加错误提示
  } finally {
    saving.value = false
  }
}

// 确认删除
const confirmDelete = () => {
  showDeleteConfirm.value = true
}

// 取消删除
const cancelDelete = () => {
  showDeleteConfirm.value = false
}

// 执行删除
const performDelete = async () => {
  if (!errorQuestion.value) return
  
  try {
    deleting.value = true
    const success = await deleteErrorQuestionMock(errorQuestion.value.id)
    
    if (success) {
      // 删除成功，返回上一页
      router.push('/manage')
    }
  } catch (error) {
    console.error('删除失败:', error)
    // 可以添加错误提示
  } finally {
    deleting.value = false
  }
}

onMounted(() => {
  loadErrorQuestion()
})
</script>

<style scoped>
.manage-detail-page {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.header-section {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
}

.back-button {
  padding: 8px 16px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.back-button:hover {
  background: var(--hover-bg);
  border-color: var(--primary-color);
}

.header-section h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: 24px;
}

.loading-state {
  text-align: center;
  padding: 60px 20px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--border-color);
  border-top: 4px solid var(--primary-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-card, .question-card, .answer-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.card-header {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.subject-tag {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
}

.question-type {
  padding: 6px 12px;
  background: var(--hover-bg);
  border-radius: 6px;
  font-size: 14px;
  color: var(--text-secondary);
}

.source-info {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.source-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.label {
  font-weight: 500;
  color: var(--text-secondary);
  font-size: 14px;
}

.value {
  color: var(--text-primary);
  font-size: 14px;
}

.question-card h3, .answer-card h3 {
  margin: 0 0 16px 0;
  color: var(--text-primary);
  font-size: 18px;
}

.question-prompt {
  font-size: 16px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre-wrap;
}

.attachments-section {
  margin-top: 20px;
}

.attachments-section h4 {
  margin: 0 0 12px 0;
  color: var(--text-primary);
  font-size: 16px;
}

.attachments-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.attachment-item {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}

.attachment-item img {
  width: 100%;
  height: auto;
  display: block;
}

.answer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.view-answer-btn {
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.view-answer-btn:hover {
  background: var(--primary-dark);
}

.answer-content {
  border-top: 1px solid var(--border-color);
  padding-top: 16px;
}

.answer-text {
  font-size: 16px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre-wrap;
  margin-bottom: 16px;
}

.analysis-section, .error-note-section {
  margin-top: 16px;
}

.analysis-section h4, .error-note-section h4 {
  margin: 0 0 8px 0;
  color: var(--text-primary);
  font-size: 16px;
}

.analysis-section p, .error-note-section p {
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-secondary);
  white-space: pre-wrap;
}

.action-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-top: 20px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.edit-btn {
  background: var(--primary-color);
  color: white;
}

.edit-btn:hover {
  background: var(--primary-dark);
}

.delete-btn {
  background: #f44336;
  color: white;
}

.delete-btn:hover {
  background: #d32f2f;
}

/* 模态框样式 */
.modal-overlay, .confirm-overlay {
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

.modal-content, .confirm-dialog {
  background: var(--card-bg);
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
}

.modal-header, .confirm-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h2, .confirm-header h3 {
  margin: 0;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--text-secondary);
}

.modal-body {
  padding: 20px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--text-primary);
}

.form-group textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-family: inherit;
  resize: vertical;
  box-sizing: border-box;
}

.form-group textarea:focus {
  outline: none;
  border-color: var(--primary-color);
}

.modal-footer, .confirm-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 20px;
  border-top: 1px solid var(--border-color);
}

.cancel-btn {
  background: var(--hover-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.cancel-btn:hover {
  background: var(--border-color);
}

.save-btn, .danger-btn {
  background: var(--primary-color);
  color: white;
}

.save-btn:hover:not(:disabled) {
  background: var(--primary-dark);
}

.danger-btn {
  background: #f44336;
}

.danger-btn:hover:not(:disabled) {
  background: #d32f2f;
}

.danger-btn:disabled, .save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.confirm-body {
  padding: 20px;
}

.confirm-body p {
  margin: 0;
  color: var(--text-primary);
  font-size: 16px;
}
</style>