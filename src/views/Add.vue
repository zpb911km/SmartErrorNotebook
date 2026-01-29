<template>
  <div class="add-page">
    <!-- 相机模态框组件 -->
    <CameraModal 
      :visible="showCamera" 
      @close="handleCameraClose" 
      @capture="handleCameraCapture" 
      @error="disableCamera"
    />

    <!-- 图片编辑模态框组件 -->
    <ImageEditor 
      :visible="showEdit" 
      :imageData="editImageData"
      @close="handleEditClose"
      @confirm="handleEditConfirm"
    />

    <div class="upload-area">
      <div class="upload-content">
        <div class="upload-icon">📷</div>
        <div class="upload-buttons">
          <div class="upload-ctn">
            <button class="upload-btn">选择文件</button>
            <input type="file" accept="image/*" @change="handleFileSelect" class="file-input" ref="fileInputRef">
          </div>
          <button class="upload-btn" @click="openCamera" :disabled="cameraDisabled" :hidden="cameraDisabled">📷拍照</button>
        </div>
      </div>
      <div class="image-preview" v-if="imageUrl">
        <img :src="imageUrl" alt="题目图片">
        <button class="remove-btn" @click="clearImage">✕</button>
      </div>
    </div>

    <div class="form-section">
      <h3>题目信息</h3>
      <div class="form-group">
        <label>科目</label>
        <SubjectSelector
          :currentSubjectId="form.subject"
          @select="(subject_id) => {form.subject = subject_id}"
        />
      </div>

      <div class="form-group">
        <label>题目</label>
        <textarea v-model="form.prompt" placeholder="请输入题目..." rows="3"></textarea>  
      </div>

      <div class="form-group">
        <label>题型</label>
        <select v-model="form.type">
          <option v-for="type in everyQuestionType" :key="type" :value="type">{{ type }}</option>
        </select>
      </div>

      <div class="form-group">
        <label>答案</label>
        <textarea v-model="form.answer" placeholder="请输入答案..." rows="3"></textarea>
      </div>

      <div class="form-group">
        <label>解析</label>
        <textarea v-model="form.analysis" placeholder="请输入解析..." rows="3"></textarea>
      </div>

      <div class="form-group">
        <label>错题小记</label>
        <textarea v-model="form.error_note" placeholder="请输入错题小记..." rows="3"></textarea>
      </div>

      <div class="form-group">
        <label>来源</label>
        <SourceSelector
          :currentSourceId="form.source"
          :subjectId="form.subject"
          @select="(source_id) => {form.source = source_id}"
        />
      </div>

      <div class="form-group">
        <label>错因</label>
        <ErrorTagSelector
          :currentErrorTagId="form.error_tag"
          @select="(error_tag_id) => {form.error_tag = error_tag_id}"
        />
      </div>

      <div class="form-group">
        <label>SRS 预设</label>
        <SRSPresetSelector
          :currentPresetId="currentPresetId"
          @select="handlePresetSelect"
        />
      </div>
    </div>

    <div class="action-buttons">
      <button class="btn cancel" @click="resetForm">取消</button>
      <button class="btn save" @click="saveError" :disabled="isSaving">保存</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import CameraModal from '../components/CameraModal.vue'
import ImageEditor from '../components/ImageEditor.vue'
import SubjectSelector from '../components/SubjectSelector.vue'
import SourceSelector from '../components/SourceSelector.vue'
import ErrorTagSelector from '../components/ErrorTagSelector.vue'
import SRSPresetSelector from '../components/SRSPresetSelector.vue'
import { QuestionType } from '../types'
import { createErrorQuestion, CreateErrorQuestionRequest } from '../apis/errorQuestions'
import { showInfo, showError } from '../utils/notification'

const imageUrl = ref('')
const isSaving = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

// 相机相关状态
const showCamera = ref(false)
const cameraDisabled = ref(false)

// 图片编辑相关状态
const showEdit = ref(false)
const editImageData = ref('')

// SRS预设相关状态
const currentPresetId = ref('')
const selectedPreset = ref(null)

// 题型
const everyQuestionType = Object.values(QuestionType)

const form = ref({
  // base info
  subject: '',
  prompt: '',
  type: '',
  answer: '',
  analysis: '',
  error_note: '',
  // source info
  source: '',
  // error tag info
  error_tag: '',
  // SRS info
  difficulty: 0,
  mastery: 0
})

onMounted(() => {
  // 获取相机权限
  navigator.mediaDevices.getUserMedia({ video: true })
    .then(() => {
      cameraDisabled.value = false
    })
    .catch(() => {
      disableCamera()
    });
})

// 处理文件选择
const handleFileSelect = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target.files && target.files[0]) {
    const file = target.files[0]
    const imageData = URL.createObjectURL(file)
    openEdit(imageData)
  }
}

// 清除图片
const clearImage = () => {
  imageUrl.value = ''
}

// 打开相机
const openCamera = () => {
  showCamera.value = true
}

// 相机关闭
const handleCameraClose = () => {
  showCamera.value = false
}

// 禁用相机
const disableCamera = () => {
  showCamera.value = false
  cameraDisabled.value = true
}

// 相机拍照
const handleCameraCapture = (imageData: string) => {
  showCamera.value = false
  openEdit(imageData)
}

// 打开图片编辑
const openEdit = (imageData: string) => {
  editImageData.value = imageData
  showEdit.value = true
}

// 图片编辑关闭
const handleEditClose = () => {
  showEdit.value = false
  editImageData.value = ''
}

// 图片编辑确认
const handleEditConfirm = (imageData: string) => {
  showEdit.value = false
  editImageData.value = ''
  imageUrl.value = imageData
}

// 处理SRS预设选择
const handlePresetSelect = (preset: any) => {
  selectedPreset.value = preset
  currentPresetId.value = preset.id
  form.value.difficulty = preset.difficulty
  form.value.mastery = preset.mastery
}

// 重置表单
const resetForm = () => {
  form.value = {
    // base info
    subject: '',
    prompt: '',
    type: '',
    answer: '',
    analysis: '',
    error_note: '',
    // source info
    source: '',
    // error tag info
    error_tag: '',
    // SRS info
    difficulty: 0,
    mastery: 0
  }
  imageUrl.value = ''
  currentPresetId.value = ''
  selectedPreset.value = null
}

// 保存错题
const saveError = async () => {
  // 验证必填字段
  if (!form.value.subject) {
    showError('错误', '请选择科目')
    return
  }
  
  if (!form.value.prompt) {
    showError('错误', '请输入题目')
    return
  }
  
  if (!form.value.type) {
    showError('错误', '请选择题型')
    return
  }
  
  isSaving.value = true
  
  try {
    // 准备请求对象
    const request: CreateErrorQuestionRequest = {
      errorQuestion: {
        userid: 'current_user_id', // 模拟当前用户ID，后端应该处理这个
        subjectid: form.value.subject,
        prompt: form.value.prompt,
        type: form.value.type as QuestionType,
        answer: form.value.answer || undefined,
        analysis: form.value.analysis || undefined,
        error_note: form.value.error_note || undefined,
      },
      srsData: {
        difficulty: form.value.difficulty,
        mastery: form.value.mastery,
        lastreviewed_at: Date.now(), // 使用当前时间戳
        review_count: 0, // 新增错题，复习次数为0
      },
      attachment: imageUrl.value ? imageUrl.value : undefined, // 图片数据，如果是base64格式
    }
    
    // 如果选择了来源，需要获取来源的详细信息
    if (form.value.source) {
      // 在实际实现中，您可能需要通过API获取来源的详细信息
      // 这里我们模拟获取来源信息
      console.log('选择了来源:', form.value.source);
    }
    
    // 如果选择了错因标签，需要获取错因标签的详细信息
    if (form.value.error_tag) {
      // 在实际实现中，您可能需要通过API获取错因标签的详细信息
      // 这里我们模拟获取错因标签信息
      console.log('选择了错因:', form.value.error_tag);
    }
    
    console.log('保存错题请求:', request)
    
    // 发送请求
    const response = await createErrorQuestion(request)
    console.log('保存错题成功:', response)
    
    showInfo('成功', '错题保存成功')
    
    // 重置表单
    resetForm()
  } catch (error) {
    console.error('保存错题失败:', error)
    showError('错误', '保存错题失败: ' + (error as Error).message)
  } finally {
    isSaving.value = false
  }
}
</script>

<style scoped>
.add-page {
  padding: 20px;
  padding-bottom: 100px;
}

.upload-ctn {
  position: relative;
}

/* 上传区域 */
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

.upload-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.upload-content p {
  color: var(--text-secondary);
  margin: 0 0 16px 0;
}

.upload-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.upload-btn {
  padding: 10px 20px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.upload-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
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