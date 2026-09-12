<template>
  <div class="add-page" :inert="isSaving">
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
          v-model="sourceSelection"
          :sources="sources"
          :subjectId="form.subject"
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
          v-model="form.note"
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
import { listSubjects as getSubjects } from '../api'
import { blobUrlToBase64 } from '../utils/attachments'
import { useQuestionEditor } from '../composables/useQuestionEditor'
import type { QuestionDraft } from '../services/questionEditor'
import { ref, watch, onMounted, onUnmounted } from 'vue'
import CameraModal from '../components/CameraModal.vue'
import ImageEditor from '../components/ImageEditor.vue'
import SubjectSelector from '../components/SubjectSelector.vue'
import SourceSelector from '../components/SourceSelector.vue'
import ErrorTagSelector from '../components/ErrorTagSelector.vue'
import {
  questionTypeLabels,
  questionTypeLabel,
  parseQuestionType
} from '../utils/questionDisplay'
import { listSources } from '../api/source'
import type { Source } from '../types/source'
import {
  selectSourceValues,
  type SourceSelection
} from '../services/sourceSelection'
import { materializeSourceSelection } from '../services/sourcePersistence'
import { showInfo, showError, showSuccess } from '../utils/notification'
import { inquiryAIAddInfo } from '../utils/inquiry'
import { llm } from '../services/llm'
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

const sources = ref<Source[]>([])
const sourceSelection = ref<SourceSelection>({ kind: 'none' })
let sourceLoadVersion = 0
onUnmounted(() => {
  ++sourceLoadVersion
})

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
const everyQuestionType = Object.values(questionTypeLabels)

const form = ref({
  // base info
  subject: '',
  prompt: '',
  type: '',
  answer: '',
  analysis: '',
  note: '',
  // source info
  source: '',
  // error tag info
  error_tags: [] as Array<{ name: string; color: string }>
})

// 从社区分享预填数据
onMounted(() => {
  const sharedData = getSharedData()
  if (sharedData) {
    form.value.prompt = sharedData.stem
    form.value.type = questionTypeLabel(sharedData.questionType)
    form.value.answer = sharedData.correctAnswer
    form.value.analysis = sharedData.explanation ?? ''
    form.value.note = sharedData.note ?? ''
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
              handleSubjectSelect(subj.id)
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
  if (form.value.subject === subjectId) return

  form.value.subject = subjectId
  form.value.source = ''
  const version = ++sourceLoadVersion
  sourceSelection.value = subjectId
    ? {
        kind: 'new',
        subjectId,
        book: null,
        chapter: null,
        knowledge: null
      }
    : { kind: 'none' }
  sources.value = []
  if (!subjectId) return

  void listSources(subjectId)
    .then((loaded) => {
      if (version !== sourceLoadVersion || form.value.subject !== subjectId)
        return
      // A save may have materialized and appended a source while this request was
      // in flight. Keep such current-only rows instead of letting a stale catalog
      // response hide them and turn the same draft back into a create operation.
      const merged = [
        ...loaded,
        ...sources.value.filter(
          (current) => !loaded.some((source) => source.id === current.id)
        )
      ]
      sources.value = merged
      const selection = sourceSelection.value
      if (selection.kind === 'new' && selection.subjectId === subjectId) {
        sourceSelection.value = selectSourceValues(
          merged,
          subjectId,
          selection.book,
          selection.chapter,
          selection.knowledge
        )
      }
    })
    .catch((error) => {
      if (version === sourceLoadVersion)
        console.error('获取来源列表失败:', error)
    })
}

// 重置表单
const editor = useQuestionEditor()
const resetForm = () => {
  editor.reset()
  form.value = {
    // base info
    subject: '',
    prompt: '',
    type: '',
    answer: '',
    analysis: '',
    note: '',
    // source info
    source: '',
    // error tag info
    error_tags: []
  }
  imageUrls.value = []
  currentPresetId.value = ''
  selectedPreset.value = null
  sourceSelection.value = { kind: 'none' }
  sources.value = []
  ++sourceLoadVersion
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
    // SourceSelector only edits a synchronous draft. Persist it exactly once at
    // the save boundary so closing/disabled UI state can never race this save.
    const materializedSource = await materializeSourceSelection(
      sourceSelection.value
    )
    sourceSelection.value = materializedSource.selection
    form.value.source = materializedSource.sourceId ?? ''
    if (
      materializedSource.source &&
      !sources.value.some(
        (source) => source.id === materializedSource.source?.id
      )
    ) {
      sources.value.push(materializedSource.source)
    }

    const draft: QuestionDraft = {
      source: sourceSelection.value,
      questionType: parseQuestionType(form.value.type),
      stem: form.value.prompt,
      correctAnswer: form.value.answer,
      explanation: form.value.analysis || null,
      note: form.value.note || null,
      tags: form.value.error_tags.map((tag) => ({ ...tag })),
      attachments: await Promise.all(
        imageUrls.value.map(async (url) => ({
          base64Data: await blobUrlToBase64(url)
        }))
      )
    }
    try {
      await editor.save(draft)
    } finally {
      sourceSelection.value = draft.source
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
    showError(
      '保存失败',
      editor.state.error ? editor.failureMessage() : String(e)
    )
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
