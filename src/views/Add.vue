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
      :autoDetect="autoDetect"
      @close="handleEditClose"
      @confirm="handleEditConfirm"
    />

    <div class="upload-area">
      <div class="upload-content">
        <Icon name="camera" :size="48" class="upload-icon" />
        <div class="upload-buttons">
          <div class="upload-ctn">
            <button class="upload-btn">选择文件</button>
            <input
              type="file"
              accept="image/*"
              @change="handleFileSelect"
              class="file-input"
              ref="fileInputRef"
            />
          </div>
          <button
            class="upload-btn"
            @click="handlePhotoClick"
            :disabled="cameraDisabled"
            :hidden="cameraDisabled"
          >
            <Icon name="camera" :size="16" /> 拍照
          </button>
        </div>
      </div>
      <div class="image-preview-list" v-if="imageUrls.length > 0">
        <div
          class="image-preview-item"
          v-for="(url, index) in imageUrls"
          :key="index"
        >
          <img :src="url" :alt="`题目图片 ${index + 1}`" />
          <div class="image-actions">
            <button
              class="action-btn edit-btn"
              @click="openEdit(url, index, true)"
              title="编辑"
            >
              <Icon name="square-pen" :size="16" />
            </button>
            <button
              class="action-btn remove-btn"
              @click="removeImage(index)"
              title="删除"
            >
              <Icon name="x" :size="16" />
            </button>
          </div>
          <div class="image-index">{{ index + 1 }}</div>
        </div>
      </div>
    </div>

    <div class="form-section">
      <div class="ai-button-container">
        <button
          v-if="!aiButtonLoading"
          @click="inquiryAI()"
          :disabled="aiButtonLoading"
          class="ai-btn"
        >
          AI 查询
        </button>
        <!-- <div class="loading-spinner" v-if="aiButtonLoading">
          <div class="spinner"></div>
        </div> -->
      </div>
      <h3>题目信息</h3>
      <div class="form-group" :class="{ loading: subjectLoading }">
        <label>科目</label>
        <SubjectSelector v-model="form.subject" @select="handleSubjectSelect" />
        <div class="loading-spinner" v-if="subjectLoading">
          <div class="spinner"></div>
        </div>
      </div>

      <div class="form-group" :class="{ loading: typeLoading }">
        <label>题型</label>
        <select v-model="form.type">
          <option v-for="type in everyQuestionType" :key="type" :value="type">
            {{ type }}
          </option>
        </select>
        <div class="loading-spinner" v-if="promptLoading">
          <div class="spinner"></div>
        </div>
      </div>

      <div class="form-group">
        <label>来源</label>
        <SourceSelector
          :currentSourceId="form.source"
          :subjectId="form.subject"
          @select="
            (source_id) => {
              form.source = source_id
              console.log('source_id:', source_id)
            }
          "
        />
      </div>

      <div class="form-group">
        <label>错因</label>
        <ErrorTagSelector
          :currentTags="form.error_tags"
          @select="
            (tags) => {
              form.error_tags = tags
            }
          "
        />
      </div>

      <div class="form-group" :class="{ loading: promptLoading }">
        <label>题目</label>
        <MarkdownTextarea
          v-model="form.prompt"
          placeholder="请输入题目..."
          rows="3"
        ></MarkdownTextarea>
        <div class="loading-spinner" v-if="promptLoading">
          <div class="spinner"></div>
        </div>
      </div>

      <div class="form-group" :class="{ loading: answerLoading }">
        <label>答案</label>
        <MarkdownTextarea
          v-model="form.answer"
          placeholder="请输入答案..."
          rows="3"
        ></MarkdownTextarea>
        <div class="loading-spinner" v-if="answerLoading">
          <div class="spinner"></div>
        </div>
      </div>

      <div class="form-group" :class="{ loading: analysisLoading }">
        <label>解析</label>
        <MarkdownTextarea
          v-model="form.analysis"
          placeholder="请输入解析..."
          rows="3"
        ></MarkdownTextarea>
        <div class="loading-spinner" v-if="analysisLoading">
          <div class="spinner"></div>
        </div>
      </div>

      <div class="form-group">
        <label>错题小记</label>
        <MarkdownTextarea
          v-model="form.error_note"
          placeholder="请输入错题小记..."
          rows="3"
        ></MarkdownTextarea>
      </div>
    </div>

    <div class="action-buttons">
      <button class="btn cancel" @click="resetForm">取消</button>
      <button class="btn save" @click="saveError" :disabled="isSaving">
        {{ isSaving ? '保存中…' : '保存' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import CameraModal from '../components/CameraModal.vue'
import ImageEditor from '../components/ImageEditor.vue'
import SubjectSelector from '../components/SubjectSelector.vue'
import SourceSelector from '../components/SourceSelector.vue'
import ErrorTagSelector from '../components/ErrorTagSelector.vue'
import { QuestionType } from '../types'
import { createErrorQuestion } from '../apis/errorQuestions'
import { createErrorTagsForQuestion } from '../apis/errorTags'
import { createSRSData } from '../apis/srsData'
import {
  createAttachmentsForQuestion,
  blobUrlToBase64
} from '../apis/attachments'
import { showInfo, showError, showSuccess } from '../utils/notification'
import { inquiryAIAddInfo } from '../utils/inquiry'
import { llm } from '../services/llm'
import { getSubjects } from '../apis'
import MarkdownTextarea from '../components/MarkdownTextarea.vue'
import { getSharedData, clearSharedData } from '../services/shareStore'

const imageUrls = ref<string[]>([])
const isSaving = ref(false)

// AI查询加载状态
const subjectLoading = ref(false)
const promptLoading = ref(false)
const typeLoading = ref(false)
const answerLoading = ref(false)
const analysisLoading = ref(false)
const aiButtonLoading = ref(false)

const selectedSource = ref<{
  book: string
  chapter: string | undefined
  knowledge: string | undefined
} | null>(null)

// 相机相关状态
const showCamera = ref(false)
const cameraDisabled = ref(false)

// 图片编辑相关状态
const showEdit = ref(false)
const editImageData = ref('')
const editingImageIndex = ref<number>(-1)
const autoDetect = ref(false)

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
  error_tags: [] as Array<{ name: string; color: string }>,
  // SRS info
  difficulty: 5
})

// 从社区分享预填数据
onMounted(() => {
  const sharedData = getSharedData()
  if (sharedData) {
    form.value.prompt = sharedData.prompt
    form.value.type = sharedData.type_
    form.value.answer = sharedData.answer
    form.value.analysis = sharedData.analysis
    form.value.error_note = sharedData.error_note
    clearSharedData()
  }
})

// 查询AI建议
const inquiryAI = async () => {
  // 重置所有加载状态
  subjectLoading.value = true
  promptLoading.value = true
  typeLoading.value = true
  answerLoading.value = true
  analysisLoading.value = true
  aiButtonLoading.value = true

  // 检查 AI 是否已配置
  if (!llm.isConfigured()) {
    showError(
      'AI 未配置',
      '请先在「设置」中配置 AI 服务（API 地址、密钥和模型）'
    )
    subjectLoading.value = false
    promptLoading.value = false
    typeLoading.value = false
    answerLoading.value = false
    analysisLoading.value = false
    aiButtonLoading.value = false
    return
  }

  // 记录是否有查询成功
  let anySuccess = false

  // 创建所有查询的Promise
  const subjectPromise = inquiryAIAddInfo(imageUrls.value, ['subject'])
    .then((result) => {
      if (result[0]?.success) anySuccess = true
      const subjectName = result[0]?.parsedContent?.subject || ''
      if (subjectName) {
        return getSubjects()
          .then((subjects) => {
            const subj = subjects.find((i) => i.name === subjectName)
            if (subj) {
              form.value.subject = subj.id
            }
            return subjectName
          })
          .catch((error) => {
            console.error('获取科目列表失败:', error)
            return ''
          })
      }
      return ''
    })
    .finally(() => {
      subjectLoading.value = false
    })

  const promptPromise = inquiryAIAddInfo(imageUrls.value, ['question_text'])
    .then((result) => {
      if (result[0]?.success) anySuccess = true
      form.value.prompt = result[0]?.parsedContent || ''
    })
    .finally(() => {
      promptLoading.value = false
    })

  const typePromise = inquiryAIAddInfo(imageUrls.value, ['question_type'])
    .then((result) => {
      if (result[0]?.success) anySuccess = true
      form.value.type = result[0]?.parsedContent?.questionType || ''
    })
    .finally(() => {
      typeLoading.value = false
    })

  const answerPromise = inquiryAIAddInfo(imageUrls.value, ['answer'])
    .then((result) => {
      if (result[0]?.success) anySuccess = true
      form.value.answer = result[0]?.parsedContent || ''
    })
    .finally(() => {
      answerLoading.value = false
    })

  const analysisPromise = inquiryAIAddInfo(imageUrls.value, ['analysis'])
    .then((result) => {
      if (result[0]?.success) anySuccess = true
      form.value.analysis = result[0]?.parsedContent || ''
    })
    .finally(() => {
      analysisLoading.value = false
    })

  try {
    // 等待所有查询完成
    await Promise.all([
      subjectPromise,
      promptPromise,
      typePromise,
      answerPromise,
      analysisPromise
    ])
    if (anySuccess) {
      console.log('AI查询完成，表单已更新:', {
        subject: form.value.subject,
        prompt: form.value.prompt,
        type: form.value.type,
        answer: form.value.answer,
        analysis: form.value.analysis
      })
      showSuccess('获取成功', '已自动填充题目信息')
    } else {
      showError('AI 查询失败', '请检查 AI 配置是否正确，或网络连接是否正常')
    }
  } catch (error) {
    console.error('AI查询失败:', error)
    showError('错误', 'AI查询失败: ' + error)
  } finally {
    aiButtonLoading.value = false
  }
}

// 处理文件选择
const handleFileSelect = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target.files && target.files[0]) {
    const file = target.files[0]
    const imageData = URL.createObjectURL(file)
    openEdit(imageData, -1, true) // -1 表示添加新图片，true 表示自动识别
  }
  // 重置 input 以便再次选择同一文件
  target.value = ''
}

// 移除图片
const removeImage = (index: number) => {
  imageUrls.value.splice(index, 1)
}

// 点击拍照按钮时打开相机
const handlePhotoClick = async () => {
  if (cameraDisabled.value) {
    return
  }

  try {
    const stream = await navigator.mediaDevices.getUserMedia({ video: true })
    stream.getTracks().forEach((track) => track.stop())
    cameraDisabled.value = false
    showCamera.value = true
  } catch {
    showError('错误', '没有相机权限')
    disableCamera()
  }
}

// 相机关闭
const handleCameraClose = () => {
  closeCameraImmediately()
}

watch(showCamera, async (visible) => {
  if (visible && cameraDisabled.value) {
    await ensureCameraAvailable()
  }
})

// 禁用相机
const disableCamera = () => {
  closeCameraImmediately()
  cameraDisabled.value = true
}

const closeCameraImmediately = () => {
  showCamera.value = false
}

const ensureCameraAvailable = async () => {
  try {
    await navigator.mediaDevices.getUserMedia({ video: true })
    cameraDisabled.value = false
  } catch {
    disableCamera()
  }
}

// 相机拍照
const handleCameraCapture = (imageData: string) => {
  closeCameraImmediately()
  openEdit(imageData, -1, true) // -1 表示添加新图片，true 表示自动识别
}

// 打开图片编辑
const openEdit = (
  imageData: string,
  index: number,
  shouldAutoDetect: boolean = false
) => {
  console.log('Add.vue openEdit, shouldAutoDetect:', shouldAutoDetect)
  editImageData.value = imageData
  editingImageIndex.value = index
  showEdit.value = true
  // 传递自动识别标志
  autoDetect.value = shouldAutoDetect
  console.log('Add.vue autoDetect.value:', autoDetect.value)
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
  if (editingImageIndex.value === -1) {
    // 添加新图片
    imageUrls.value.push(imageData)
  } else {
    // 替换指定索引的图片
    imageUrls.value[editingImageIndex.value] = imageData
  }
  editingImageIndex.value = -1
}

// 处理科目选择
const handleSubjectSelect = (subjectId: string) => {
  console.log('handleSubjectSelect', subjectId)
  form.value.subject = subjectId
  form.value.source = ''
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
    error_tags: [],
    // SRS info
    difficulty: 5
  }
  imageUrls.value = []
  currentPresetId.value = ''
  selectedPreset.value = null
  selectedSource.value = null
}

// 保存错题
const saveError = async () => {
  // 防止重复提交
  if (isSaving.value) return

  // 验证必填字段
  if (!form.value.subject) {
    // 弹窗询问是否继续
    if (!confirm('您未选择科目，是否继续保存？')) {
      return
    }
  }

  if (!form.value.prompt) {
    showError('错误', '请输入题目')
    return
  }

  isSaving.value = true
  console.log('开始保存错题，图片数量:', imageUrls.value.length)

  try {
    // 1. 创建错题
    console.log('正在创建错题...')
    const errorQuestion = await createErrorQuestion({
      user_id: 'current_user', // TODO: 从用户状态获取
      subject_id: form.value.subject,
      source_id: form.value.source || undefined,
      prompt: form.value.prompt,
      type: form.value.type as QuestionType,
      answer: form.value.answer || undefined,
      analysis: form.value.analysis || undefined,
      error_note: form.value.error_note || undefined
    })
    console.log('错题创建成功, id:', errorQuestion.id)

    // 2. 批量创建错因标签
    if (form.value.error_tags.length > 0) {
      await createErrorTagsForQuestion(errorQuestion.id, form.value.error_tags)
    }

    // 3. 创建SRS数据（失败不阻塞保存流程，可在复习时重新生成）
    try {
      await createSRSData(errorQuestion.id, form.value.difficulty)
      console.log('SRS数据创建成功')
    } catch (srsErr) {
      console.warn('SRS数据创建失败（不影响错题保存）:', srsErr)
    }

    // 4. 批量上传图片
    if (imageUrls.value.length > 0) {
      console.log('正在处理图片上传...')
      const attachmentsData = await Promise.all(
        imageUrls.value.map(async (url, index) => {
          try {
            const base64Data = await blobUrlToBase64(url)
            console.log(`图片 ${index + 1} 转换成功, 长度:`, base64Data?.length || 0)
            return {
              question_id: errorQuestion.id,
              type_: 'original',
              file_type: 'image',
              base64_data: base64Data
            }
          } catch (error) {
            console.error(`转换图片 ${index + 1} 失败:`, error)
            throw error
          }
        })
      )

      console.log('正在调用后端保存图片...')
      await createAttachmentsForQuestion(errorQuestion.id, attachmentsData)
      console.log('图片保存完成')
    } else {
      console.log('没有图片需要保存')
    }
    // 保存成功后重置表单
    const savedImgCount = imageUrls.value.length
    const savedTagCount = form.value.error_tags.length
    resetForm()
    showInfo(
      '错题添加成功',
      `已保存 ${savedImgCount} 张错题图片${savedTagCount > 0 ? `，${savedTagCount} 个错因标签` : ''}`
    )
  } catch (e) {
    console.error('保存错题失败:', e)
    showError('保存失败', '请检查网络连接后重试')
  } finally {
    isSaving.value = false
  }
}
</script>

<style scoped>
.add-page {
  padding: 40px 20px;
  background: var(--bg-primary);
  min-height: 100vh;
  margin: 0 auto;
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

.image-preview-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
  margin-top: 20px;
}

.image-preview-item {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  background: var(--input-bg);
}

.image-preview-item img {
  width: 100%;
  height: 200px;
  object-fit: cover;
  display: block;
}

.image-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 8px;
  opacity: 0;
  transition: opacity 0.3s;
}

.image-preview-item:hover .image-actions {
  opacity: 1;
}

.action-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  color: white;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s;
}

.action-btn.edit-btn {
  background: rgba(0, 0, 0, 0.6);
}

.action-btn.edit-btn:hover {
  background: var(--primary-color);
}

.action-btn.remove-btn {
  background: rgba(220, 53, 69, 0.8);
}

.action-btn.remove-btn:hover {
  background: #dc3545;
}

.image-index {
  position: absolute;
  bottom: 8px;
  left: 8px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
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

.form-group.loading {
  position: relative;
  pointer-events: none; /* 防止用户交互 */
}

.spinner {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  border: 4px solid rgba(0, 0, 0, 0.1);
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border-left-color: #09f;
  animation: spin 0.5s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
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

.ai-button-container {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-top: 24px;
}
.ai-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  background-color: var(--primary-color);
  color: white;
  font-size: 16px;
  cursor: pointer;
  transition:
    background-color 0.3s,
    transform 0.3s;
}
.ai-btn:hover {
  background-color: var(--primary-dark);
}
.ai-btn:active {
  transform: scale(0.98);
}

/* ===== 暗色模式适配 ===== */
body.dark-theme .spinner {
  border-color: rgba(255, 255, 255, 0.15);
  border-left-color: #4fc3f7;
}

body.dark-theme .btn.cancel {
  border: 1px solid var(--border-color);
}

body.dark-theme .upload-area.drag-over {
  background: rgba(25, 118, 210, 0.2);
}
</style>
