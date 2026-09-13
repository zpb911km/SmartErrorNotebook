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
      :image-data="editImageData"
      :auto-detect="autoDetect"
      @close="handleEditClose"
      @confirm="handleEditConfirm"
    />

    <q-card flat bordered class="upload-area">
      <div class="upload-content">
        <Icon name="camera" :size="48" class="upload-icon" />
        <div class="upload-buttons">
          <div class="upload-ctn">
            <q-btn
              no-caps
              unelevated
              type="button"
              color="primary"
              class="upload-btn"
            >
              选择文件
            </q-btn>
            <input
              ref="fileInputRef"
              type="file"
              accept="image/*"
              class="file-input"
              @change="handleFileSelect"
            />
          </div>
          <q-btn
            no-caps
            unelevated
            type="button"
            color="primary"
            class="upload-btn"
            :disable="cameraDisabled"
            :hidden="cameraDisabled"
            @click="handlePhotoClick"
          >
            <Icon name="camera" :size="16" /> 拍照
          </q-btn>
        </div>
      </div>
      <div v-if="imageUrls.length > 0" class="image-preview-list">
        <div
          v-for="(url, index) in imageUrls"
          :key="index"
          class="image-preview-item"
        >
          <img :src="url" :alt="`题目图片 ${index + 1}`" />
          <div class="image-actions">
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              aria-label="编辑"
              class="action-btn edit-btn"
              title="编辑"
              @click="openEdit(url, index, true)"
            >
              <Icon name="square-pen" :size="16" />
            </q-btn>
            <q-btn
              no-caps
              unelevated
              type="button"
              color="negative"
              aria-label="删除"
              class="action-btn remove-btn"
              title="删除"
              @click="removeImage(index)"
            >
              <Icon name="x" :size="16" />
            </q-btn>
          </div>
          <div class="image-index">
            {{ index + 1 }}
          </div>
        </div>
      </div>
    </q-card>

    <q-card flat bordered class="form-section">
      <div class="ai-button-container">
        <q-btn
          no-caps
          unelevated
          type="button"
          color="primary"
          :loading="aiButtonLoading"
          :disable="aiButtonLoading"
          class="ai-btn"
          @click="inquiryAI()"
        >
          AI 识别
        </q-btn>
        <!-- <div class="loading-spinner" v-if="aiButtonLoading">
          <div class="spinner"></div>
        </div> -->
      </div>
      <h3>题目信息</h3>
      <div class="form-group" :class="{ loading: subjectLoading }">
        <label>科目</label>
        <SubjectSelector v-model="form.subject" @select="handleSubjectSelect" />
        <q-spinner v-if="subjectLoading" color="primary" size="28px" />
      </div>

      <div class="form-group" :class="{ loading: typeLoading }">
        <label>题型</label>
        <q-select
          v-model="form.type"
          outlined
          dense
          emit-value
          map-options
          :options="everyQuestionType"
          aria-label="选择选项"
        />
        <q-spinner v-if="typeLoading" color="primary" size="28px" />
      </div>

      <div class="form-group">
        <label>来源</label>
        <SourceSelector
          v-model="sourceSelection"
          :sources="sources"
          :subject-id="form.subject"
        />
      </div>

      <div class="form-group">
        <label>错因</label>
        <ErrorTagSelector
          :current-tags="form.error_tags"
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
          default-view-mode="edit"
          placeholder="请输入题目..."
          rows="3"
        />
        <q-spinner v-if="promptLoading" color="primary" size="28px" />
      </div>

      <div class="form-group" :class="{ loading: answerLoading }">
        <label>答案</label>
        <MarkdownTextarea
          v-model="form.answer"
          default-view-mode="edit"
          placeholder="请输入答案..."
          rows="3"
        />
        <q-spinner v-if="answerLoading" color="primary" size="28px" />
      </div>

      <div class="form-group" :class="{ loading: analysisLoading }">
        <label>解析</label>
        <MarkdownTextarea
          v-model="form.analysis"
          default-view-mode="edit"
          placeholder="请输入解析..."
          rows="3"
        />
        <q-spinner v-if="analysisLoading" color="primary" size="28px" />
      </div>

      <div class="form-group">
        <label>错题小记</label>
        <MarkdownTextarea
          v-model="form.note"
          default-view-mode="edit"
          placeholder="请输入错题小记..."
          rows="3"
        />
      </div>
    </q-card>

    <div class="action-buttons">
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="btn cancel"
        @click="resetForm"
      >
        取消
      </q-btn>
      <q-btn
        no-caps
        unelevated
        type="button"
        color="primary"
        class="btn save"
        :disable="isSaving"
        @click="saveError"
      >
        {{ isSaving ? '保存中…' : '保存' }}
      </q-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'

import { listSubjects as getSubjects } from '../api'
import { listSources } from '../api/source'
import CameraModal from '../components/CameraModal.vue'
import ErrorTagSelector from '../components/ErrorTagSelector.vue'
import ImageEditor from '../components/ImageEditor.vue'
import MarkdownTextarea from '../components/MarkdownTextarea.vue'
import SourceSelector from '../components/SourceSelector.vue'
import SubjectSelector from '../components/SubjectSelector.vue'
import { useLatestRequest } from '../composables/useLatestRequest'
import { useQuestionEditor } from '../composables/useQuestionEditor'
import { llm } from '../services/llm'
import type { QuestionDraft } from '../services/questionEditor'
import { clearSharedData, getSharedData } from '../services/shareStore'
import { materializeSourceSelection } from '../services/sourcePersistence'
import {
  selectSourceValues,
  type SourceSelection
} from '../services/sourceSelection'
import type { Source } from '../types/source'
import { blobUrlToBase64 } from '../utils/attachments'
import { confirmAction } from '../utils/dialog'
import { inquiryAIAddInfo } from '../utils/inquiry'
import { showError, showInfo, showSuccess } from '../utils/notification'
import {
  parseQuestionType,
  questionTypeLabel,
  questionTypeLabels
} from '../utils/questionDisplay'

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

// Ignore late results after navigation/reset and preserve edits made during recognition.
const beginAIRequest = useLatestRequest()
const inquiryAI = async () => {
  if (aiButtonLoading.value || isSaving.value) return
  if (!llm.isConfigured()) {
    showError(
      'AI 未配置',
      '请先在「设置」中配置 AI 服务（API 地址、密钥和模型）'
    )
    return
  }
  if (!imageUrls.value.length) {
    showInfo('请先添加图片', 'AI 识别需要题目图片')
    return
  }
  const isCurrent = beginAIRequest()
  const snapshot = { ...form.value }
  const images = [...imageUrls.value]
  const canApply = () =>
    isCurrent() &&
    !isSaving.value &&
    images.length === imageUrls.value.length &&
    images.every((url, i) => url === imageUrls.value[i])
  const fields = [
    { tag: 'question_text', key: 'prompt', loading: promptLoading },
    { tag: 'question_type', key: 'type', loading: typeLoading },
    { tag: 'answer', key: 'answer', loading: answerLoading },
    { tag: 'analysis', key: 'analysis', loading: analysisLoading }
  ] as const
  aiButtonLoading.value = true
  subjectLoading.value = true
  for (const field of fields) field.loading.value = true
  let applied = 0
  try {
    const results = await Promise.allSettled([
      ...fields.map(async ({ tag, key, loading }) => {
        try {
          const [result] = await inquiryAIAddInfo(images, [tag])
          const parsed = result?.parsedContent
          const value =
            key === 'type'
              ? parsed && typeof parsed === 'object' && 'questionType' in parsed
                ? parsed.questionType
                : undefined
              : parsed
          if (
            canApply() &&
            result?.success &&
            typeof value === 'string' &&
            value.trim() &&
            form.value[key] === snapshot[key]
          ) {
            if (key === 'type' && !everyQuestionType.includes(value)) return
            form.value[key] = value
            applied++
          }
        } finally {
          if (isCurrent()) loading.value = false
        }
      }),
      (async () => {
        try {
          const [result] = await inquiryAIAddInfo(images, ['subject'])
          const parsed = result?.parsedContent
          const name =
            parsed && typeof parsed === 'object' && 'subject' in parsed
              ? parsed.subject
              : undefined
          if (!result?.success || typeof name !== 'string' || !canApply())
            return
          const subjects = await getSubjects()
          const subject = subjects.find((item) => item.name === name)
          if (
            subject &&
            canApply() &&
            form.value.subject === snapshot.subject
          ) {
            await handleSubjectSelect(subject.id)
            applied++
          }
        } finally {
          if (isCurrent()) subjectLoading.value = false
        }
      })()
    ])
    if (!canApply()) return
    if (applied)
      showSuccess(
        '识别完成',
        '已填充识别结果；识别期间的手动修改已保留，请核对后保存。'
      )
    else
      showInfo(
        '未填充内容',
        '未取得有效结果，或对应内容已被手动修改。请检查 AI 配置与图片。'
      )
    if (results.some((result) => result.status === 'rejected'))
      showError('部分识别失败', '可重试识别，已填写的内容仍然保留。')
  } finally {
    if (isCurrent()) aiButtonLoading.value = false
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
    const stream = await navigator.mediaDevices.getUserMedia({ video: true })
    stream.getTracks().forEach((track) => track.stop())
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
  editImageData.value = imageData
  editingImageIndex.value = index
  showEdit.value = true
  // 传递自动识别标志
  autoDetect.value = shouldAutoDetect
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
  beginAIRequest()
  for (const loading of [
    subjectLoading,
    promptLoading,
    typeLoading,
    answerLoading,
    analysisLoading,
    aiButtonLoading
  ])
    loading.value = false
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
  isSaving.value = true
  try {
    // 验证必填字段
    if (!form.value.subject) {
      // 弹窗询问是否继续
      if (!(await confirmAction('您未选择科目，是否继续保存？'))) {
        return
      }
    }

    if (!form.value.prompt) {
      showError('错误', '请输入题目')
      return
    }

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
