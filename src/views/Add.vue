<template>
  <div class="add-page">
    <div class="upload-area" :class="{ 'drag-over': isDragOver }" @dragover.prevent="isDragOver = true" @dragleave.prevent="isDragOver = false" @drop.prevent="handleDrop">
      <div class="upload-content" v-if="!imageUrl">
        <div class="upload-icon">📷</div>
        <p>点击或拖拽上传题目图片</p>
        <input type="file" accept="image/*" @change="handleFileSelect" class="file-input">
      </div>
      <div class="image-preview" v-else>
        <img :src="imageUrl" alt="题目图片">
        <button class="remove-btn" @click="clearImage">✕</button>
      </div>
    </div>

    <div class="form-section">
      <h3>题目信息</h3>
      <div class="form-group">
        <label>科目</label>
        <select v-model="form.subject">
          <option value="">请选择科目</option>
          <option value="math">数学</option>
          <option value="physics">物理</option>
          <option value="chemistry">化学</option>
          <option value="english">英语</option>
          <option value="chinese">语文</option>
        </select>
      </div>

      <div class="form-group">
        <label>知识点</label>
        <input type="text" v-model="form.knowledge" placeholder="例如：函数、力学、语法等">
      </div>

      <div class="form-group">
        <label>题型</label>
        <select v-model="form.type">
          <option value="">请选择题型</option>
          <option value="choice">选择题</option>
          <option value="fill">填空题</option>
          <option value="calc">计算题</option>
          <option value="essay">论述题</option>
        </select>
      </div>

      <div class="form-group">
        <label>难度</label>
        <div class="difficulty-selector">
          <button v-for="level in difficultyLevels" :key="level.value"
                  class="difficulty-btn"
                  :class="{ active: form.difficulty === level.value }"
                  @click="form.difficulty = level.value">
            {{ level.label }}
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>错误原因</label>
        <select v-model="form.reason">
          <option value="">请选择错误原因</option>
          <option value="careless">粗心大意</option>
          <option value="concept">概念不清</option>
          <option value="method">方法错误</option>
          <option value="calculation">计算错误</option>
          <option value="other">其他</option>
        </select>
      </div>

      <div class="form-group">
        <label>备注</label>
        <textarea v-model="form.note" placeholder="添加备注或笔记..." rows="3"></textarea>
      </div>
    </div>

    <div class="ai-suggestion" v-if="aiResult">
      <h3>AI 分析结果</h3>
      <div class="suggestion-item">
        <span class="suggestion-label">识别科目：</span>
        <span class="suggestion-value">{{ aiResult.subject }}</span>
      </div>
      <div class="suggestion-item">
        <span class="suggestion-label">知识点：</span>
        <span class="suggestion-value">{{ aiResult.knowledge }}</span>
      </div>
      <div class="suggestion-item">
        <span class="suggestion-label">错因分析：</span>
        <span class="suggestion-value">{{ aiResult.reason }}</span>
      </div>
      <div class="suggestion-item">
        <span class="suggestion-label">解题建议：</span>
        <span class="suggestion-value">{{ aiResult.suggestion }}</span>
      </div>
    </div>

    <div class="action-buttons">
      <button class="btn cancel" @click="resetForm">取消</button>
      <button class="btn save" @click="saveError">保存</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const isDragOver = ref(false)
const imageUrl = ref('')

const form = ref({
  subject: '',
  knowledge: '',
  type: '',
  difficulty: 'medium',
  reason: '',
  note: ''
})

const difficultyLevels = [
  { label: '简单', value: 'easy' },
  { label: '中等', value: 'medium' },
  { label: '困难', value: 'hard' }
]

const aiResult = ref({
  subject: '数学',
  knowledge: '函数',
  reason: '概念不清',
  suggestion: '建议复习函数的定义域和值域相关概念'
})

const handleFileSelect = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target.files && target.files[0]) {
    const file = target.files[0]
    imageUrl.value = URL.createObjectURL(file)
  }
}

const handleDrop = (e: DragEvent) => {
  isDragOver.value = false
  if (e.dataTransfer?.files && e.dataTransfer.files[0]) {
    const file = e.dataTransfer.files[0]
    imageUrl.value = URL.createObjectURL(file)
  }
}

const clearImage = () => {
  imageUrl.value = ''
}

const resetForm = () => {
  form.value = {
    subject: '',
    knowledge: '',
    type: '',
    difficulty: 'medium',
    reason: '',
    note: ''
  }
  imageUrl.value = ''
}

const saveError = () => {
  console.log('保存错题', form.value)
  alert('错题保存成功！')
  resetForm()
}
</script>

<style scoped>
.add-page {
  padding: 20px;
  padding-bottom: 100px;
}

.upload-area {
  background: var(--card-bg);
  border: 2px dashed var(--border-color);
  border-radius: 12px;
  padding: 40px 20px;
  text-align: center;
  margin-bottom: 24px;
  transition: all 0.3s;
}

.upload-area.drag-over {
  border-color: var(--primary-color);
  background: var(--primary-light);
}

.upload-content {
  position: relative;
}

.upload-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.upload-content p {
  color: var(--text-secondary);
  margin: 0;
}

.file-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.image-preview {
  position: relative;
}

.image-preview img {
  max-width: 100%;
  max-height: 300px;
  border-radius: 8px;
}

.remove-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.form-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 24px;
}

.form-section h3 {
  font-size: 16px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.form-group {
  margin-bottom: 16px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.form-group input,
.form-group select,
.form-group textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  background: var(--input-bg);
  color: var(--text-primary);
  box-sizing: border-box;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  outline: none;
  border-color: var(--primary-color);
}

.difficulty-selector {
  display: flex;
  gap: 8px;
}

.difficulty-btn {
  flex: 1;
  padding: 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.difficulty-btn.active {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.ai-suggestion {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 24px;
  color: white;
}

.ai-suggestion h3 {
  font-size: 16px;
  margin: 0 0 12px 0;
}

.suggestion-item {
  display: flex;
  margin-bottom: 8px;
  font-size: 14px;
}

.suggestion-label {
  font-weight: 500;
  margin-right: 8px;
  opacity: 0.9;
}

.suggestion-value {
  flex: 1;
  opacity: 1;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
}

.btn.cancel {
  background: var(--card-bg);
  color: var(--text-primary);
}

.btn.save {
  background: var(--primary-color);
  color: white;
}

.btn:active {
  transform: scale(0.98);
}
</style>