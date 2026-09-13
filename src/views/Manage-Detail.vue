<template>
  <div class="manage-detail-page" :inert="saving">
    <div v-if="detailLoadState === 'error'" role="alert">
      详情加载失败，请重试。
      <q-btn no-caps unelevated type="button" flat @click="fetchErrorDetail">
        重新加载
      </q-btn>
    </div>
    <q-btn
      v-if="editor.state.cleanupIds.length"
      no-caps
      unelevated
      type="button"
      flat
      :disable="editor.busy.value || saving"
      @click="retryAttachmentCleanup"
    >
      题目已保存，重试清理旧附件
    </q-btn>
    <!-- 顶部导航栏 -->
    <div class="detail-header">
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="back-btn"
        @click="goBack"
      >
        <Icon name="arrow-left" :size="16" class="back-icon" />
        <span>返回</span>
      </q-btn>
      <h2>错题详情管理</h2>
      <div class="header-actions">
        <q-btn
          no-caps
          unelevated
          type="button"
          flat
          class="action-btn edit-btn glare-btn"
          :disable="saving || detailLoadState !== 'ready'"
          @click="toggleEditMode"
        >
          <Icon name="square-pen" :size="16" class="btn-icon" />
          <span class="btn-label">{{ isEditing ? '取消编辑' : '编辑' }}</span>
        </q-btn>
        <q-btn
          no-caps
          unelevated
          type="button"
          color="negative"
          class="action-btn delete-btn glare-btn"
          @click="confirmDelete"
        >
          <Icon name="trash-2" :size="16" class="btn-icon" />
          <span class="btn-label">删除</span>
        </q-btn>
      </div>
    </div>

    <p v-if="detailLoadState === 'loading'" role="status">正在加载详情…</p>
    <div v-else-if="detailLoadState === 'error'" role="alert">
      详情未完整加载，暂时无法编辑。
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        :disable="saving"
        @click="fetchErrorDetail"
      >
        重试加载
      </q-btn>
    </div>

    <div v-if="errorDetail" class="detail-content">
      <!-- 基本信息区域 -->
      <q-card flat bordered class="info-section">
        <div class="section-title">基本信息</div>

        <!-- 科目选择 -->
        <div class="form-group">
          <label>科目</label>
          <SubjectSelector
            :model-value="editForm.subjectId"
            :disabled="!isEditing"
            @select="handleSubjectSelect"
          />
        </div>

        <!-- 来源信息 -->
        <div class="form-group">
          <label>来源信息</label>
          <SourceSelector
            v-model="sourceSelection"
            :disable="!isEditing || saving"
            :sources="sources"
            :subject-id="editForm.subjectId"
          />
        </div>

        <!-- 题型 -->
        <div class="form-group">
          <label>题型</label>
          <q-select
            v-model="editForm.type"
            outlined
            dense
            emit-value
            map-options
            :options="[
              { label: '单选题', value: '单选题' },
              { label: '多选题', value: '多选题' },
              { label: '填空题', value: '填空题' },
              { label: '简答题', value: '简答题' },
              { label: '论述题', value: '论述题' },
              { label: '计算题', value: '计算题' },
              { label: '判断题', value: '判断题' },
              { label: '其他', value: '' }
            ]"
            aria-label="选择选项"
            :disable="!isEditing"
            class="form-select"
          />
        </div>
      </q-card>

      <!-- 题目内容区域 -->
      <q-card flat bordered class="content-section">
        <div class="section-title">题目内容</div>

        <!-- 题目图片展示 -->
        <div
          v-if="
            (isEditing ? tempQuestionImages.length : questionImages.length) > 0
          "
          class="images-gallery"
        >
          <div class="gallery-title">题目图片（点击预览）</div>
          <div class="image-grid">
            <div
              v-for="(image, index) in isEditing
                ? tempQuestionImages
                : questionImages"
              :key="image.id || `temp-${index}`"
              class="image-item"
              @click="previewImage(image)"
            >
              <img
                :src="buildImageSrc(image)"
                :alt="'题目图片'"
                class="question-image"
              />
              <q-btn
                v-if="isEditing"
                no-caps
                unelevated
                type="button"
                color="negative"
                aria-label="删除图片"
                class="delete-image-btn"
                title="删除图片"
                @click.stop="deleteTempImage(image)"
              >
                <Icon name="x" :size="16" />
              </q-btn>
            </div>
          </div>
        </div>

        <!-- 添加图片按钮（仅编辑模式） -->
        <div v-if="isEditing" class="upload-section">
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="btn-add-images"
            @click="triggerImageUpload"
          >
            <Icon name="camera" :size="16" /> 添加图片
          </q-btn>
          <input
            ref="imageInput"
            type="file"
            accept="image/*"
            multiple
            style="display: none"
            @change="handleImageSelect"
          />
        </div>

        <div class="form-group">
          <!-- <label>题干</label> -->
          <MarkdownTextarea
            v-if="isEditing"
            v-model="editForm.prompt"
            :show-preview="true"
            :default-view-mode="'edit'"
            preview-title="题目预览"
          />
          <MarkdownTextarea
            v-else
            :model-value="editForm.prompt"
            :show-preview="true"
            :default-view-mode="'preview'"
            preview-title=""
            :textarea-class="'readonly-textarea'"
            readonly
          />
        </div>
      </q-card>

      <!-- 答案区域 -->
      <q-card flat bordered class="answer-section">
        <div class="section-title">参考答案</div>

        <div class="form-group">
          <!-- <label>参考答案</label> -->
          <MarkdownTextarea
            v-if="isEditing"
            v-model="editForm.answer"
            :show-preview="true"
            :default-view-mode="'edit'"
            preview-title="答案预览"
          />
          <MarkdownTextarea
            v-else
            :model-value="editForm.answer"
            :show-preview="true"
            :default-view-mode="'preview'"
            preview-title=""
            :textarea-class="'readonly-textarea'"
            readonly
          />
        </div>
      </q-card>

      <!-- 解析区域 -->
      <q-card flat bordered class="analysis-section">
        <div class="section-title">解析</div>

        <div class="form-group">
          <!-- <label>解析</label> -->
          <MarkdownTextarea
            v-if="isEditing"
            v-model="editForm.analysis"
            :show-preview="true"
            :default-view-mode="'edit'"
            preview-title="解析预览"
          />
          <MarkdownTextarea
            v-else
            :model-value="editForm.analysis"
            :show-preview="true"
            :default-view-mode="'preview'"
            preview-title=""
            :textarea-class="'readonly-textarea'"
            readonly
          />
        </div>
      </q-card>

      <!-- 错因标签区域 -->
      <q-card flat bordered class="tags-section">
        <div class="section-title">错因标签</div>

        <!-- 非编辑模式：只显示标签 -->
        <div v-if="!isEditing" class="tags-display">
          <span
            v-for="tag in filteredErrorTags"
            :key="tag.id"
            class="tag-item"
            :style="{ backgroundColor: tag.color + '20', color: tag.color }"
          >
            {{ tag.name }}
          </span>
          <span v-if="filteredErrorTags.length === 0" class="no-tags"
            >暂无标签</span
          >
        </div>

        <!-- 编辑模式：使用标签选择器 -->
        <div v-else class="tags-edit">
          <ErrorTagSelector
            :current-tags="tempErrorTags"
            @select="
              (tags) => {
                tempErrorTags = tags
              }
            "
          />
        </div>
      </q-card>

      <!-- 错题笔记区域 -->
      <q-card flat bordered class="note-section">
        <div class="section-title">错题笔记</div>

        <div class="form-group">
          <!-- <label>笔记内容</label> -->
          <MarkdownTextarea
            v-if="isEditing"
            v-model="editForm.note"
            :show-preview="true"
            :default-view-mode="'edit'"
            preview-title="笔记预览"
          />
          <MarkdownTextarea
            v-else
            :model-value="editForm.note"
            :show-preview="true"
            :default-view-mode="'preview'"
            preview-title=""
            :textarea-class="'readonly-textarea'"
            readonly
          />
        </div>
      </q-card>

      <!-- SRS 数据展示 -->
      <q-card flat bordered class="srs-section">
        <div class="section-title">学习数据</div>

        <div v-if="srsData" class="srs-stats">
          <div class="stat-item">
            <span class="stat-label">掌握程度</span>
            <span class="stat-value">{{ calculateMastery(srsData) }}%</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">难度系数</span>
            <span class="stat-value">{{ srsData.difficulty.toFixed(2) }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">复习次数</span>
            <span class="stat-value">{{ srsData.reviewCount }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">最后复习</span>
            <span class="stat-value">{{
              formatTimestamp(srsData.lastReviewAt)
            }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">稳定性</span>
            <span class="stat-value"
              >{{ srsData.stability.toFixed(2) }} 天</span
            >
          </div>
          <div class="stat-item">
            <span class="stat-label">召回率</span>
            <span class="stat-value"
              >{{ (srsData.retrievability * 100).toFixed(1) }}%</span
            >
          </div>
        </div>
        <div v-else class="no-srs-data">
          <p>暂无学习数据</p>
        </div>
      </q-card>

      <!-- 时间信息 -->
      <q-card flat bordered class="time-section">
        <div class="section-title">时间信息</div>
        <div class="time-info">
          <div class="time-item">
            <span class="time-label">创建时间：</span>
            <span class="time-value">{{
              formatTimestamp(errorDetail.createdAt)
            }}</span>
          </div>
          <div class="time-item">
            <span class="time-label">更新时间：</span>
            <span class="time-value">{{
              formatTimestamp(errorDetail.updatedAt)
            }}</span>
          </div>
        </div>
      </q-card>
    </div>

    <!-- 加载状态 -->
    <div v-else-if="detailLoadState === 'loading'" class="loading-state">
      <q-spinner color="primary" size="28px" />
      <p>加载中...</p>
    </div>

    <!-- 确认删除弹窗 -->
    <q-dialog
      class="notebook-dialog"
      persistent
      :model-value="showDeleteConfirm"
    >
      <div class="modal-content confirm-modal">
        <div class="modal-header">
          <h3>确认删除</h3>
        </div>
        <div class="modal-body">
          <p>确定要删除这道错题吗？此操作不可恢复。</p>
        </div>
        <div class="modal-footer">
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="btn-cancel"
            @click="showDeleteConfirm = false"
          >
            取消
          </q-btn>
          <q-btn
            no-caps
            unelevated
            type="button"
            color="primary"
            class="btn-confirm"
            @click="deleteError"
          >
            确认删除
          </q-btn>
        </div>
      </div>
    </q-dialog>

    <!-- 保存按钮 -->
    <div v-if="isEditing" class="save-bar">
      <q-btn
        no-caps
        unelevated
        type="button"
        color="primary"
        class="save-btn glare-btn"
        :disable="saving || detailLoadState !== 'ready'"
        @click="saveChanges"
      >
        {{ saving ? '保存中...' : '保存修改' }}
      </q-btn>
    </div>

    <!-- 非编辑模式：简单图片预览 -->
    <ImagePreview
      v-if="!isEditing"
      :visible="showImagePreview"
      :image-url="previewImageUrl"
      @close="closeImagePreview"
    />

    <!-- 编辑模式：图片编辑器预览 -->
    <ImageEditor
      v-if="isEditing"
      :visible="showImagePreview"
      :image-data="previewImageUrl"
      :auto-detect="false"
      @close="closeImagePreview"
      @confirm="handlePreviewConfirm"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { deleteQuestion as removeQuestion, getAttachment } from '../api'
import ErrorTagSelector from '../components/ErrorTagSelector.vue'
import ImageEditor from '../components/ImageEditor.vue'
import ImagePreview from '../components/ImagePreview.vue'
import MarkdownTextarea from '../components/MarkdownTextarea.vue'
import SourceSelector from '../components/SourceSelector.vue'
import SubjectSelector from '../components/SubjectSelector.vue'
import { useQuestionEditor } from '../composables/useQuestionEditor'
import type { QuestionDraft } from '../services/questionEditor'
import { loadQuestionDetail } from '../services/questionQueries'
import { materializeSourceSelection } from '../services/sourcePersistence'
import {
  selectSourceValues,
  type SourceSelection
} from '../services/sourceSelection'
import type { SrsData } from '../types'
import type { Attachment, Subject, Tag as ErrorTagType } from '../types'
import type { QuestionView } from '../types/questionView'
import type { Source } from '../types/source'
import { buildDataUrl, fileToBase64 } from '../utils/attachments'
import { showAlert } from '../utils/dialog'
import {
  formatTimestamp as formatDateTime,
  parseQuestionType,
  questionTypeLabel
} from '../utils/questionDisplay'

const router = useRouter()
const route = useRoute()

// 错题ID
const errorId = computed(() => route.params.id as string)

// 数据状态
const errorDetail = ref<QuestionView | null>(null)
const subjects = ref<Subject[]>([])
const errorTags = ref<ErrorTagType[]>([])
const srsData = ref<SrsData | null>(null)
const questionImages = ref<Attachment[]>([])
const sources = ref<Source[]>([])
const sourceSelection = ref<SourceSelection>({ kind: 'none' })

// 图片上传相关
const imageInput = ref<HTMLInputElement | null>(null)

// 临时图片列表（用于编辑时的暂存）
const tempQuestionImages = ref<Attachment[]>([])

// 临时标签列表（用于编辑时的暂存）
const tempErrorTags = ref<Array<{ name: string; color: string }>>([])

// 过滤掉已删除的标签
const filteredErrorTags = computed(() => {
  return errorTags.value.filter((tag) => !tag.name.startsWith('[已删除]'))
})

// 编辑状态
const isEditing = ref(false)
const saving = ref(false)

const editForm = ref({
  subjectId: '',
  sourceId: '',
  prompt: '',
  type: '',
  answer: '',
  analysis: '',
  note: ''
})

// 弹窗状态
const showDeleteConfirm = ref(false)

const detailLoadState = ref<'loading' | 'ready' | 'error'>('loading')
let detailLoadVersion = 0

// Publish editable data only after every required resource has loaded.
const fetchErrorDetail = async () => {
  const version = ++detailLoadVersion
  detailLoadState.value = 'loading'
  try {
    const {
      question,
      sources: loadedSources,
      subjects: loadedSubjects
    } = await loadQuestionDetail(errorId.value)
    const attachments = await Promise.all(
      question.attachmentIds.map(getAttachment)
    )
    if (version !== detailLoadVersion) return false

    errorDetail.value = question
    subjects.value = loadedSubjects
    sources.value = loadedSources
    sourceSelection.value = question.sourceId
      ? { kind: 'existing', sourceId: question.sourceId }
      : selectSourceValues(
          loadedSources,
          question.subject?.id ?? '',
          null,
          null,
          null
        )
    editForm.value = {
      subjectId: question.subject?.id ?? '',
      sourceId: question.sourceId ?? '',
      prompt: question.stem,
      type: questionTypeLabel(question.questionType),
      answer: question.correctAnswer ?? '',
      analysis: question.explanation ?? '',
      note: question.note ?? ''
    }
    errorTags.value = question.tags
    questionImages.value = attachments
    srsData.value = question.srs
    detailLoadState.value = 'ready'
    return true
  } catch (error) {
    if (version === detailLoadVersion) detailLoadState.value = 'error'
    console.error('获取错题详情失败:', error)
    return false
  }
}

// 切换编辑模式
const toggleEditMode = () => {
  if (saving.value || detailLoadState.value !== 'ready') return
  editor.reset()
  if (isEditing.value) {
    // 取消编辑，恢复原值

    if (errorDetail.value) {
      editForm.value = {
        subjectId: errorDetail.value.subject?.id ?? '',
        sourceId: errorDetail.value.sourceId || '',
        prompt: errorDetail.value.stem,
        type: questionTypeLabel(errorDetail.value.questionType),
        answer: errorDetail.value.correctAnswer || '',
        analysis: errorDetail.value.explanation || '',
        note: errorDetail.value.note || ''
      }
      sourceSelection.value = errorDetail.value.sourceId
        ? { kind: 'existing', sourceId: errorDetail.value.sourceId }
        : selectSourceValues(
            sources.value,
            errorDetail.value.subject?.id ?? '',
            null,
            null,
            null
          )
    }

    // 清空临时数据（不需要重新加载，因为原始数据还在）
    tempQuestionImages.value = []
    tempErrorTags.value = []
  } else {
    // 进入编辑模式，初始化临时列表（使用深拷贝避免引用污染）
    tempQuestionImages.value = questionImages.value.map((img) => ({ ...img }))
    // 初始化临时标签列表
    tempErrorTags.value = errorTags.value.map((tag) => ({
      name: tag.name,
      color: tag.color
    }))
  }
  isEditing.value = !isEditing.value
}

// 保存修改
const editor = useQuestionEditor()
const retryAttachmentCleanup = async () => {
  await editor.retryCleanup()
  showAlert(
    editor.state.cleanupIds.length
      ? '部分附件仍未能清理，请重试。'
      : '附件清理完成。'
  )
}
const saveChanges = async () => {
  if (
    !errorDetail.value ||
    saving.value ||
    editor.busy.value ||
    detailLoadState.value !== 'ready'
  )
    return

  saving.value = true

  try {
    // Persist the synchronous selector draft before updating the question. Once
    // created, retain its ID so a failed question save can be retried without
    // creating a duplicate source.
    const materializedSource = await materializeSourceSelection(
      sourceSelection.value
    )
    sourceSelection.value = materializedSource.selection
    editForm.value.sourceId = materializedSource.sourceId ?? ''
    if (
      materializedSource.source &&
      !sources.value.some(
        (source) => source.id === materializedSource.source?.id
      )
    ) {
      sources.value.push(materializedSource.source)
    }

    const draft: QuestionDraft = {
      id: errorId.value,
      source: sourceSelection.value,
      questionType: parseQuestionType(editForm.value.type),
      stem: editForm.value.prompt,
      correctAnswer: editForm.value.answer,
      explanation: editForm.value.analysis || null,
      note: editForm.value.note || null,
      tags: tempErrorTags.value.map((tag) => ({ ...tag })),
      attachments: tempQuestionImages.value.map((image) => {
        const original = questionImages.value.find(
          (item) => item.id === image.id
        )
        return {
          id: original?.base64Data === image.base64Data ? image.id : undefined,
          base64Data: image.base64Data
        }
      })
    }
    try {
      await editor.save(
        draft,
        questionImages.value.map((image) => image.id)
      )
    } finally {
      sourceSelection.value = draft.source
    }
    const refreshed = await fetchErrorDetail()
    isEditing.value = false
    tempQuestionImages.value = []
    showAlert(
      editor.state.cleanupIds.length
        ? '题目已保存，部分旧附件清理失败，可单独重试清理。'
        : refreshed
          ? '保存成功！'
          : '题目已保存，但详情刷新失败，请重试加载。'
    )
  } catch (error) {
    console.error('保存失败:', error)
    showAlert(editor.state.error ? editor.failureMessage() : String(error))
  } finally {
    saving.value = false
  }
}

// 确认删除
const confirmDelete = () => {
  showDeleteConfirm.value = true
}

// 计算掌握程度
const calculateMastery = (srs: SrsData | null): number => {
  if (!srs) return 0

  const reviewCount = srs.reviewCount || 0
  const stability = srs.stability || 0
  const recallRate = srs.retrievability || 0

  // 综合计算掌握程度（0-100%）
  const reviewScore = Math.min(reviewCount / 10, 1) * 100
  const stabilityScore = Math.min(stability / 30, 1) * 100
  const recallScore = recallRate * 100

  return Math.round(
    reviewScore * 0.3 + stabilityScore * 0.3 + recallScore * 0.4
  )
}

// 删除错题
const deleteError = async () => {
  try {
    await removeQuestion(errorId.value)
    showDeleteConfirm.value = false
    // 返回管理页面
    router.push('/manage')
  } catch (error) {
    console.error('删除失败:', error)
    showAlert('删除失败，请重试')
  }
}

// 格式化时间戳
const formatTimestamp = (timestamp?: string | null) =>
  formatDateTime(timestamp, '未知')

// 返回上一页
const goBack = () => {
  router.back()
}

// 处理科目选择
const handleSubjectSelect = (subjectId: string) => {
  if (editForm.value.subjectId === subjectId) return
  editForm.value.subjectId = subjectId
  editForm.value.sourceId = ''
  sourceSelection.value = selectSourceValues(
    sources.value,
    subjectId,
    null,
    null,
    null
  )
}

// 构建图片src
const buildImageSrc = (attachment: Attachment) =>
  buildDataUrl(attachment.base64Data, attachment.mimeType)

const showImagePreview = ref(false)
const previewImageUrl = ref('')
const editingImageId = ref<string | null>(null) // 当前正在编辑的图片ID

// 关闭图片预览
const closeImagePreview = () => {
  showImagePreview.value = false
  previewImageUrl.value = ''
  editingImageId.value = null // 清除编辑中的图片ID
}

// 处理预览确认（保存编辑后的图片）
const handlePreviewConfirm = (imageData: string) => {
  if (!editingImageId.value) {
    console.error('没有正在编辑的图片ID')
    closeImagePreview()
    return
  }

  // 查找并更新临时图片列表中的对应图片
  const imageIndex = tempQuestionImages.value.findIndex(
    (img) => img.id === editingImageId.value
  )

  if (imageIndex !== -1) {
    // 将 base64 数据转换为纯 base64 字符串（去掉 data:image/jpeg;base64, 前缀）
    const base64Data = imageData.split(',')[1] || imageData

    // 更新图片的 base64Data
    tempQuestionImages.value[imageIndex].base64Data = base64Data
    tempQuestionImages.value[imageIndex].mimeType =
      imageData.match(/^data:([^;]+);/)?.[1] ?? 'image/jpeg'
  } else {
    console.error('❌ 未找到对应的图片ID:', editingImageId.value)
    console.error(
      '当前所有图片ID:',
      tempQuestionImages.value.map((img) => img.id)
    )
  }

  // 关闭预览
  closeImagePreview()
}

// 预览图片 - 使用 ImageEditor 组件
const previewImage = (attachment: Attachment) => {
  const imageUrl = buildImageSrc(attachment)

  if (!imageUrl) {
    console.error('图片URL为空，无法预览')
    return
  }

  previewImageUrl.value = imageUrl
  editingImageId.value = attachment.id // 记录当前编辑的图片ID
  showImagePreview.value = true
}

// 触发图片选择
const triggerImageUpload = () => {
  if (imageInput.value) {
    imageInput.value.click()
  }
}

// 处理图片选择 - 只添加到临时列表
const handleImageSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const files = target.files

  if (!files || files.length === 0) {
    return
  }

  try {
    // 创建临时的 Attachment 对象用于显示
    for (let i = 0; i < files.length; i++) {
      const file = files[i]

      // 将文件转换为 base64
      const base64Data = await fileToBase64(file)

      const tempAttachment: Attachment = {
        id: `temp-${Date.now()}-${i}`,
        mimeType: file.type || 'image/png',
        base64Data: base64Data, // 保存 base64 数据
        sha256: ''
      }
      tempQuestionImages.value.push(tempAttachment)

      // 如果是第一张图片，自动打开编辑器
      if (i === 0) {
        previewImageUrl.value = `data:${file.type};base64,${base64Data}`
        editingImageId.value = tempAttachment.id
        showImagePreview.value = true
      }
    }
  } catch (error) {
    console.error('处理图片失败:', error)
  } finally {
    // 清空 input，允许重复选择同一文件
    if (target) {
      target.value = ''
    }
  }
}

// 删除临时图片
const deleteTempImage = (image: Attachment) => {
  // 从临时列表中移除
  tempQuestionImages.value = tempQuestionImages.value.filter(
    (img) => img.id !== image.id
  )
}

onMounted(fetchErrorDetail)
watch(errorId, fetchErrorDetail)
onUnmounted(() => {
  ++detailLoadVersion
})
</script>
