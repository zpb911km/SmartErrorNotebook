<template>
  <div class="manage-detail-page">
    <!-- 顶部导航栏 -->
    <div class="detail-header" ref="detailHeaderRef">
      <button class="back-btn" v-ripple @click="goBack">
        <Icon name="arrow-left" :size="16" class="back-icon" />
        <span>返回</span>
      </button>
      <h2>错题详情管理</h2>
      <div
        class="header-actions"
        ref="headerActionsRef"
        :class="{ collapsed: actionsCollapsed }"
      >
        <button
          v-ripple
          class="action-btn edit-btn glare-btn"
          @click="toggleEditMode"
          :disabled="saving || detailLoadState !== 'ready'"
        >
          <Icon name="square-pen" :size="16" class="btn-icon" />
          <span class="btn-label">{{ isEditing ? '取消编辑' : '编辑' }}</span>
        </button>
        <button
          v-ripple
          class="action-btn delete-btn glare-btn"
          @click="confirmDelete"
        >
          <Icon name="trash-2" :size="16" class="btn-icon" />
          <span class="btn-label">删除</span>
        </button>
      </div>
    </div>

    <p v-if="detailLoadState === 'loading'" role="status">正在加载详情…</p>
    <div v-else-if="detailLoadState === 'error'" role="alert">
      详情未完整加载，暂时无法编辑。
      <button :disabled="saving" @click="fetchErrorDetail">重试加载</button>
    </div>

    <div v-if="errorDetail" class="detail-content">
      <!-- 基本信息区域 -->
      <div class="info-section">
        <div class="section-title">基本信息</div>

        <!-- 科目选择 -->
        <div class="form-group">
          <label>科目</label>
          <SubjectSelector
            :modelValue="editForm.subject_id"
            :disabled="!isEditing"
            @select="handleSubjectSelect"
          />
        </div>

        <!-- 来源信息 -->
        <div class="form-group">
          <label>来源信息</label>
          <SourceSelector
            v-model="sourceSelection"
            :disable="!isEditing || sourceSelectorDisabled"
            :sources="sources"
            :subjectId="editForm.subject_id"
          />
        </div>

        <!-- 题型 -->
        <div class="form-group">
          <label>题型</label>
          <select
            v-model="editForm.type"
            :disabled="!isEditing"
            class="form-select"
          >
            <option value="单选题">单选题</option>
            <option value="多选题">多选题</option>
            <option value="填空题">填空题</option>
            <option value="简答题">简答题</option>
            <option value="论述题">论述题</option>
            <option value="计算题">计算题</option>
            <option value="判断题">判断题</option>
            <option value="">其他</option>
          </select>
        </div>
      </div>

      <!-- 题目内容区域 -->
      <div class="content-section">
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
              <button
                v-if="isEditing"
                class="delete-image-btn"
                @click.stop="deleteTempImage(image)"
                title="删除图片"
              >
                <Icon name="x" :size="16" />
              </button>
            </div>
          </div>
        </div>

        <!-- 添加图片按钮（仅编辑模式） -->
        <div v-if="isEditing" class="upload-section">
          <button class="btn-add-images" @click="triggerImageUpload">
            <Icon name="camera" :size="16" /> 添加图片
          </button>
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
      </div>

      <!-- 答案区域 -->
      <div class="answer-section">
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
      </div>

      <!-- 解析区域 -->
      <div class="analysis-section">
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
      </div>

      <!-- 错因标签区域 -->
      <div class="tags-section">
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
            :currentTags="tempErrorTags"
            @select="
              (tags) => {
                tempErrorTags = tags
              }
            "
          />
        </div>
      </div>

      <!-- 错题笔记区域 -->
      <div class="note-section">
        <div class="section-title">错题笔记</div>

        <div class="form-group">
          <!-- <label>笔记内容</label> -->
          <MarkdownTextarea
            v-if="isEditing"
            v-model="editForm.error_note"
            :show-preview="true"
            :default-view-mode="'edit'"
            preview-title="笔记预览"
          />
          <MarkdownTextarea
            v-else
            :model-value="editForm.error_note"
            :show-preview="true"
            :default-view-mode="'preview'"
            preview-title=""
            :textarea-class="'readonly-textarea'"
            readonly
          />
        </div>
      </div>

      <!-- SRS 数据展示 -->
      <div class="srs-section">
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
            <span class="stat-value">{{ srsData.review_count }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">最后复习</span>
            <span class="stat-value">{{
              formatTimestamp(srsData.last_review_at)
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
              >{{ (srsData.recall_rate * 100).toFixed(1) }}%</span
            >
          </div>
        </div>
        <div v-else class="no-srs-data">
          <p>暂无学习数据</p>
        </div>
      </div>

      <!-- 时间信息 -->
      <div class="time-section">
        <div class="section-title">时间信息</div>
        <div class="time-info">
          <div class="time-item">
            <span class="time-label">创建时间：</span>
            <span class="time-value">{{
              formatTimestamp(errorDetail.created_at)
            }}</span>
          </div>
          <div class="time-item">
            <span class="time-label">更新时间：</span>
            <span class="time-value">{{
              formatTimestamp(errorDetail.updated_at)
            }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-else class="loading-state">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 确认删除弹窗 -->
    <div v-if="showDeleteConfirm" class="modal-overlay">
      <div class="modal-content confirm-modal">
        <div class="modal-header">
          <h3>确认删除</h3>
        </div>
        <div class="modal-body">
          <p>确定要删除这道错题吗？此操作不可恢复。</p>
        </div>
        <div class="modal-footer">
          <button
            v-ripple
            class="btn-cancel"
            @click="showDeleteConfirm = false"
          >
            取消
          </button>
          <button v-ripple class="btn-confirm" @click="deleteError">
            确认删除
          </button>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div v-if="isEditing" class="save-bar">
      <button
        v-ripple
        class="save-btn glare-btn"
        @click="saveChanges"
        :disabled="saving || detailLoadState !== 'ready'"
      >
        {{ saving ? '保存中...' : '保存修改' }}
      </button>
    </div>

    <!-- 非编辑模式：简单图片预览 -->
    <ImagePreview
      v-if="!isEditing"
      :visible="showImagePreview"
      :imageUrl="previewImageUrl"
      @close="closeImagePreview"
    />

    <!-- 编辑模式：图片编辑器预览 -->
    <ImageEditor
      v-if="isEditing"
      :visible="showImagePreview"
      :imageData="previewImageUrl"
      :autoDetect="false"
      @close="closeImagePreview"
      @confirm="handlePreviewConfirm"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { marked } from 'marked'
import markedKatex from 'marked-katex-extension'
import 'katex/dist/katex.min.css'

// 配置 marked 支持 KaTeX
marked.use(
  markedKatex({
    throwOnError: false,
    output: 'html',
    nonStandard: true
  })
)

import {
  buildDataUrl,
  fileToBase64,
  removeQuestion,
  getAttachmentsForQuestion,
  getErrorTagsForQuestion,
  getQuestionForUi,
  getQuestionSrs,
  getSubjects,
  saveQuestionAggregate,
  CompatibilityOperationError
} from '../api/compat'
import { listSources } from '../api/source'
import type { Source } from '../types/source'
import {
  selectSourceValues,
  type SourceSelection
} from '../services/sourceSelection'
import { materializeSourceSelection } from '../services/sourcePersistence'
import type {
  ErrorQuestion,
  Subject,
  ErrorTags as ErrorTagType,
  Attachment
} from '../types/legacy'
import SourceSelector from '../components/SourceSelector.vue'
import SubjectSelector from '../components/SubjectSelector.vue'
import MarkdownTextarea from '../components/MarkdownTextarea.vue'
import ImageEditor from '../components/ImageEditor.vue'
import ImagePreview from '../components/ImagePreview.vue'
import ErrorTagSelector from '../components/ErrorTagSelector.vue'

const router = useRouter()
const route = useRoute()

// 错题ID
const errorId = computed(() => route.params.id as string)

// 数据状态
const errorDetail = ref<
  (ErrorQuestion & { created_at?: number; updated_at?: number }) | null
>(null)
const subjects = ref<Subject[]>([])
const errorTags = ref<ErrorTagType[]>([])
const srsData = ref<any>(null)
const questionImages = ref<Attachment[]>([])
const answerImages = ref<Attachment[]>([])
const sources = ref<Source[]>([])
const sourceSelection = ref<SourceSelection>({ kind: 'none' })

// 图片上传相关
const imageInput = ref<HTMLInputElement | null>(null)

// 临时图片列表（用于编辑时的暂存）
const tempQuestionImages = ref<Attachment[]>([])
const imagesToDelete = ref<string[]>([]) // 待删除的图片ID列表
const imagesToAdd = ref<File[]>([]) // 待添加的文件列表

// 临时标签列表（用于编辑时的暂存）
const tempErrorTags = ref<Array<{ name: string; color: string }>>([])

// 过滤掉已删除的标签
const filteredErrorTags = computed(() => {
  return errorTags.value.filter((tag) => !tag.name.startsWith('[已删除]'))
})

// 编辑状态
const isEditing = ref(false)
const saving = ref(false)
const sourceSelectorDisabled = ref(false) // 控制SourceSelector的启用状态

// 按钮自适应：空间不足时折叠文字只留图标
const headerActionsRef = ref<HTMLElement | null>(null)
const detailHeaderRef = ref<HTMLElement | null>(null)
const actionsCollapsed = ref(false)
const fullButtonsWidth = ref(0) // 挂载时测量的按钮全宽（含文字）
const backBtnWidth = ref(0) // 挂载时测量的返回按钮宽度
const titleMinWidth = ref(0) // 挂载时测量的标题最小宽度
let actionsObserver: ResizeObserver | null = null
const editForm = ref({
  subject_id: '',
  source_id: '',
  prompt: '',
  type: '',
  answer: '',
  analysis: '',
  error_note: ''
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
    const [question, tags, attachments, srs, loadedSources] = await Promise.all([
      getQuestionForUi(errorId.value),
      getErrorTagsForQuestion(errorId.value),
      getAttachmentsForQuestion(errorId.value),
      getQuestionSrs(errorId.value).catch(() => null),
      listSources()
    ])
    if (version !== detailLoadVersion) return false

    errorDetail.value = question
    sources.value = loadedSources
    sourceSelection.value = question.source_id
      ? { kind: 'existing', sourceId: question.source_id }
      : selectSourceValues(loadedSources, question.subject_id, null, null, null)
    editForm.value = {
      subject_id: question.subject_id,
      source_id: question.source_id ?? '',
      prompt: question.prompt,
      type: question.type,
      answer: question.answer ?? '',
      analysis: question.analysis ?? '',
      error_note: question.error_note ?? ''
    }
    errorTags.value = tags
    questionImages.value = attachments.filter((item) => item.type_ === 'original')
    answerImages.value = attachments.filter((item) => item.type_ === 'answer')
    srsData.value = srs
    detailLoadState.value = 'ready'
    return true
  } catch (error) {
    if (version === detailLoadVersion) detailLoadState.value = 'error'
    console.error('获取错题详情失败:', error)
    return false
  }
}

// 获取科目列表
const fetchSubjects = async () => {
  try {
    subjects.value = await getSubjects()
  } catch (error) {
    console.error('获取科目列表失败:', error)
  }
}

// 切换编辑模式
const toggleEditMode = () => {
  if (saving.value || detailLoadState.value !== 'ready') return
  if (isEditing.value) {
    // 取消编辑，恢复原值
    console.log('取消编辑，恢复原始状态...')

    if (errorDetail.value) {
      editForm.value = {
        subject_id: errorDetail.value.subject_id,
        source_id: errorDetail.value.source_id || '',
        prompt: errorDetail.value.prompt,
        type: (errorDetail.value as any).type_ || errorDetail.value.type,
        answer: errorDetail.value.answer || '',
        analysis: errorDetail.value.analysis || '',
        error_note: errorDetail.value.error_note || ''
      }
      sourceSelection.value = errorDetail.value.source_id
        ? { kind: 'existing', sourceId: errorDetail.value.source_id }
        : selectSourceValues(
            sources.value,
            errorDetail.value.subject_id,
            null,
            null,
            null
          )
    }

    // 清空临时数据（不需要重新加载，因为原始数据还在）
    tempQuestionImages.value = []
    imagesToDelete.value = []
    imagesToAdd.value = []
    tempErrorTags.value = []

    console.log('已恢复原始状态')
  } else {
    // 进入编辑模式，初始化临时列表（使用深拷贝避免引用污染）
    tempQuestionImages.value = questionImages.value.map((img) => ({ ...img }))
    imagesToDelete.value = []
    imagesToAdd.value = []
    // 初始化临时标签列表
    tempErrorTags.value = errorTags.value.map((tag) => ({
      name: tag.name,
      color: tag.color
    }))
  }
  isEditing.value = !isEditing.value
}

// 保存修改
const saveChanges = async () => {
  if (!errorDetail.value || saving.value || detailLoadState.value !== 'ready') return

  saving.value = true
  sourceSelectorDisabled.value = true

  try {
    // Persist the synchronous selector draft before updating the question. Once
    // created, retain its ID so a failed question save can be retried without
    // creating a duplicate source.
    const materializedSource = await materializeSourceSelection(
      sourceSelection.value
    )
    sourceSelection.value = materializedSource.selection
    editForm.value.source_id = materializedSource.sourceId ?? ''
    if (
      materializedSource.source &&
      !sources.value.some((source) => source.id === materializedSource.source?.id)
    ) {
      sources.value.push(materializedSource.source)
    }

    // Compatibility bridge for the current aggregate-shaped UI only. Questions,
    // tags and attachments have independent lifecycles and this sequence is not
    // transactional. A future UI should call the Current APIs per resource and
    // expose separate saving/error/retry state instead of one aggregate request.
    await saveQuestionAggregate(
      { id: errorId.value, ...editForm.value },
      tempErrorTags.value,
      tempQuestionImages.value,
      questionImages.value
    )

    const refreshed = await fetchErrorDetail()
    isEditing.value = false
    tempQuestionImages.value = []
    imagesToDelete.value = []
    imagesToAdd.value = []
    alert(refreshed ? '保存成功！' : '题目已保存，但详情刷新失败，请稍后刷新页面。')
  } catch (error) {
    console.error('保存失败:', error)
    if (error instanceof CompatibilityOperationError && error.committed) {
      const refreshed = await fetchErrorDetail()
      isEditing.value = false
      tempQuestionImages.value = []
      imagesToDelete.value = []
      imagesToAdd.value = []
      alert(refreshed
        ? '题目已保存，但部分旧附件未能清理。'
        : '题目已保存，但部分旧附件未能清理，详情刷新也失败。请重试加载。')
    } else {
      alert('保存失败，请重试')
    }
  } finally {
    saving.value = false
    sourceSelectorDisabled.value = false
  }
}

// 确认删除
const confirmDelete = () => {
  showDeleteConfirm.value = true
}

// 计算掌握程度
const calculateMastery = (srs: any): number => {
  if (!srs) return 0

  const reviewCount = srs.review_count || 0
  const stability = srs.stability || 0
  const recallRate = srs.recall_rate || 0

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
    alert('删除失败，请重试')
  }
}

// 格式化时间戳
const formatTimestamp = (timestamp?: number) => {
  if (!timestamp) return '未知'
  const date = new Date(timestamp * 1000)
  return date.toLocaleString('zh-CN')
}

// 返回上一页
const goBack = () => {
  router.back()
}

// 处理科目选择
const handleSubjectSelect = (subjectId: string) => {
  if (editForm.value.subject_id === subjectId) return
  editForm.value.subject_id = subjectId
  editForm.value.source_id = ''
  sourceSelection.value = selectSourceValues(
    sources.value,
    subjectId,
    null,
    null,
    null
  )
}

// 构建图片src
const buildImageSrc = (attachment: any) => {
  console.log('构建图片URL:', attachment)
  try {
    // 如果有 base64_data，优先使用（可能是编辑后的数据）
    if (attachment.base64_data && attachment.base64_data.length > 0) {
      // 根据文件类型确定MIME类型
      let mimeType = 'image/png'
      if (attachment.file_type === 'jpeg' || attachment.file_type === 'jpg') {
        mimeType = 'image/jpeg'
      } else if (attachment.file_type === 'webp') {
        mimeType = 'image/webp'
      } else if (attachment.file_type === 'gif') {
        mimeType = 'image/gif'
      }

      return buildDataUrl(attachment.base64_data, mimeType)
    }

    // 如果没有 base64_data，但有原始文件引用（仅用于未编辑的临时图片）
    if (attachment._file) {
      return URL.createObjectURL(attachment._file)
    }

    // 根据文件类型确定MIME类型
    let mimeType = 'image/png'
    if (attachment.file_type === 'jpeg' || attachment.file_type === 'jpg') {
      mimeType = 'image/jpeg'
    } else if (attachment.file_type === 'webp') {
      mimeType = 'image/webp'
    } else if (attachment.file_type === 'gif') {
      mimeType = 'image/gif'
    }

    // 使用 base64ToBlobUrl 或构建 data URL
    return buildDataUrl(attachment.base64_data, mimeType)
  } catch (error) {
    console.error('构建图片URL失败:', error)
    return ''
  }
}

// 图片预览状态
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
  console.log('========== 图片编辑确认 ==========')
  console.log('editingImageId:', editingImageId.value)
  console.log('imageData 类型:', typeof imageData)
  console.log('imageData 前100字符:', imageData.substring(0, 100))
  console.log('imageData 长度:', imageData.length)
  console.log('tempQuestionImages 数量:', tempQuestionImages.value.length)
  console.log(
    'tempQuestionImages IDs:',
    tempQuestionImages.value.map((img) => img.id)
  )

  if (!editingImageId.value) {
    console.error('没有正在编辑的图片ID')
    closeImagePreview()
    return
  }

  // 查找并更新临时图片列表中的对应图片
  const imageIndex = tempQuestionImages.value.findIndex(
    (img) => img.id === editingImageId.value
  )

  console.log('找到的索引:', imageIndex)

  if (imageIndex !== -1) {
    console.log('找到对应的图片，索引:', imageIndex)
    console.log(
      '原始 base64_data 长度:',
      tempQuestionImages.value[imageIndex].base64_data?.length || 0
    )
    console.log(
      '原始 file_type:',
      tempQuestionImages.value[imageIndex].file_type
    )

    // 将 base64 数据转换为纯 base64 字符串（去掉 data:image/jpeg;base64, 前缀）
    const base64Data = imageData.split(',')[1] || imageData

    console.log('新的 base64_data 长度:', base64Data.length)
    console.log('新数据前缀:', imageData.substring(0, 30))

    // 更新图片的 base64_data
    tempQuestionImages.value[imageIndex].base64_data = base64Data

    console.log('✅ 更新成功！')
    console.log(
      '更新后 base64_data 长度:',
      tempQuestionImages.value[imageIndex].base64_data.length
    )
    console.log('更新后的图片对象:', {
      id: tempQuestionImages.value[imageIndex].id,
      base64_data_length:
        tempQuestionImages.value[imageIndex].base64_data.length,
      file_type: tempQuestionImages.value[imageIndex].file_type
    })
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
const previewImage = (attachment: any) => {
  console.log('预览图片:', attachment)
  const imageUrl = buildImageSrc(attachment)

  if (!imageUrl) {
    console.error('图片URL为空，无法预览')
    return
  }

  console.log('预览图片URL:', imageUrl.substring(0, 50) + '...')
  previewImageUrl.value = imageUrl
  editingImageId.value = attachment.id // 记录当前编辑的图片ID
  showImagePreview.value = true
}

// 触发图片选择
const triggerImageUpload = () => {
  console.log('触发图片选择')
  if (imageInput.value) {
    imageInput.value.click()
  }
}

// 处理图片选择 - 只添加到临时列表
const handleImageSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const files = target.files

  if (!files || files.length === 0) {
    console.log('未选择文件')
    return
  }

  console.log('选择了', files.length, '个文件')

  try {
    // 将文件添加到待添加列表
    for (let i = 0; i < files.length; i++) {
      imagesToAdd.value.push(files[i])
      console.log(`添加文件到临时列表:`, files[i].name)
    }

    // 创建临时的 Attachment 对象用于显示
    for (let i = 0; i < files.length; i++) {
      const file = files[i]

      // 将文件转换为 base64
      const base64Data = await fileToBase64(file)

      const tempAttachment: any = {
        id: `temp-${Date.now()}-${i}`,
        question_id: errorId.value,
        type_: 'original',
        file_type: file.type.split('/')[1] || 'png',
        base64_data: base64Data, // 保存 base64 数据
        name: file.name,
        _file: file // 保存原始文件引用
      }
      tempQuestionImages.value.push(tempAttachment)

      // 如果是第一张图片，自动打开编辑器
      if (i === 0) {
        console.log('自动打开图片编辑器')
        previewImageUrl.value = `data:${file.type};base64,${base64Data}`
        editingImageId.value = tempAttachment.id
        showImagePreview.value = true
      }
    }

    console.log('临时图片列表:', tempQuestionImages.value.length, '个')
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
const deleteTempImage = (image: any) => {
  console.log('删除临时图片:', image.id)

  // 如果是已有图片（有真实ID），加入待删除列表
  if (image.id && !image.id.startsWith('temp-')) {
    imagesToDelete.value.push(image.id)
    console.log('标记为待删除:', image.id)
  }

  // 从临时列表中移除
  tempQuestionImages.value = tempQuestionImages.value.filter(
    (img) => img.id !== image.id
  )
  console.log('删除成功，剩余图片:', tempQuestionImages.value.length, '个')
}

onMounted(() => {
  fetchSubjects()

  // 按钮自适应：等数据加载完成后再测量并启动观察
  const setupAdaptive = () => {
    nextTick(() => {
      if (headerActionsRef.value) {
        fullButtonsWidth.value = headerActionsRef.value.scrollWidth
      }
      if (detailHeaderRef.value) {
        const back = detailHeaderRef.value.querySelector('.back-btn')
        if (back) backBtnWidth.value = (back as HTMLElement).offsetWidth
        const title = detailHeaderRef.value.querySelector('h2')
        if (title) titleMinWidth.value = (title as HTMLElement).scrollWidth
      }

      actionsObserver = new ResizeObserver(([entry]) => {
        const totalWidth = entry.target.clientWidth
        const totalNeeded =
          backBtnWidth.value + fullButtonsWidth.value + titleMinWidth.value + 16

        if (actionsCollapsed.value) {
          if (totalWidth >= totalNeeded + 30) {
            actionsCollapsed.value = false
          }
        } else {
          if (totalWidth < totalNeeded - 2) {
            actionsCollapsed.value = true
          }
        }
      })
      if (detailHeaderRef.value) {
        actionsObserver.observe(detailHeaderRef.value)
        // 初始检测：页面加载完立即判断，不等 resize
        const initWidth = detailHeaderRef.value.clientWidth
        const initNeeded =
          backBtnWidth.value + fullButtonsWidth.value + titleMinWidth.value + 16
        if (initWidth < initNeeded - 2) {
          actionsCollapsed.value = true
        }
      }
    })
  }

  // 等异步数据全部到位后再测量
  fetchErrorDetail().then(setupAdaptive)
})

onUnmounted(() => {
  actionsObserver?.disconnect()
})
</script>

<style scoped>
.manage-detail-page {
  padding: 20px;
  padding-bottom: 200px;
  background: var(--bg-primary);
  min-height: 100vh;
  width: 100%;
}

/* 桌面端优化 - 全屏显示 */
@media (min-width: 769px) {
  .manage-detail-page {
    max-width: none;
    margin: 0;
    padding: 40px;
    padding-bottom: 200px;
  }
}

/* 顶部导航 */
.detail-header {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
  gap: 8px;
}

.detail-header h2 {
  grid-column: 2;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin: 0;
  font-size: 20px;
  color: var(--text-primary);
}

.back-btn {
  grid-column: 1;
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  padding: 8px 16px;
  border-radius: 8px;
  transition: all 0.2s ease;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.back-icon {
  width: 16px;
  height: 16px;
  transition: transform 0.2s ease;
}

.back-btn:hover .back-icon {
  transform: translateX(-3px);
}

.back-btn:hover {
  background: var(--primary-light);
  border-color: var(--primary-color);
  color: var(--primary-color);
  transform: translateX(-2px);
  box-shadow: 0 2px 8px rgba(25, 118, 210, 0.15);
}

.back-btn:active {
  transform: translateX(0);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.header-actions {
  grid-column: 3;
  justify-self: end;
  display: flex;
  gap: 6px;
  min-width: 0;
  overflow: hidden;
  flex-wrap: nowrap;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
  white-space: nowrap;
  gap: 4px;
}

.btn-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
}

.btn-label {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  flex: 0 1 auto;
  min-width: 0;
  margin-left: 6px;
}

/* 空间不足时隐藏文字，只留图标 */
.header-actions.collapsed .action-btn {
  padding: 8px 6px;
}

.header-actions.collapsed .btn-label {
  display: none;
}

.edit-btn {
  background: var(--primary-color);
  color: white;
}

.edit-btn:hover {
  background: #1565c0;
}

.delete-btn {
  background: #f44336;
  color: white;
}

.delete-btn:hover {
  background: #d32f2f;
}

/* 内容区域 */
.detail-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.info-section,
.content-section,
.answer-section,
.analysis-section,
.tags-section,
.note-section,
.srs-section,
.time-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.06),
    0 1px 2px rgba(0, 0, 0, 0.04);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.info-section:hover,
.content-section:hover,
.answer-section:hover,
.analysis-section:hover,
.tags-section:hover,
.note-section:hover,
.srs-section:hover,
.time-section:hover {
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.08),
    0 2px 8px rgba(0, 0, 0, 0.06);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 2px solid var(--primary-light);
}

/* 表单元素 */
.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  font-weight: 500;
}

.form-select,
.form-textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 14px;
  box-sizing: border-box;
  transition: border-color 0.2s;
}

.form-select:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--primary-color);
}

.form-select:disabled,
.form-textarea:disabled {
  background: var(--bg-secondary);
  cursor: not-allowed;
  opacity: 0.7;
}

/* Markdown 预览样式 */
.markdown-preview {
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.6;
  word-break: break-word;
  min-height: 100px;
}

.markdown-preview :deep(h1),
.markdown-preview :deep(h2),
.markdown-preview :deep(h3),
.markdown-preview :deep(h4),
.markdown-preview :deep(h5),
.markdown-preview :deep(h6) {
  margin: 0.8em 0 0.4em;
  font-weight: 600;
}

.markdown-preview :deep(p) {
  margin: 0.5em 0;
}

.markdown-preview :deep(code) {
  background: rgba(25, 118, 210, 0.12);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.markdown-preview :deep(pre) {
  background: var(--code-bg, #0f172a);
  padding: 10px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.markdown-preview :deep(pre code) {
  background: transparent;
  padding: 0;
  color: var(--code-text, #e2e8f0);
}

.markdown-preview :deep(pre code.hljs) {
  background: transparent;
  color: var(--code-text, #e2e8f0);
}

.markdown-preview :deep(ul),
.markdown-preview :deep(ol) {
  padding-left: 20px;
  margin: 0.5em 0;
}

.markdown-preview :deep(blockquote) {
  margin: 0.5em 0;
  padding-left: 10px;
  border-left: 3px solid var(--border-color);
  color: var(--text-secondary);
}

.markdown-preview :deep(a) {
  color: var(--primary-color);
  text-decoration: underline;
}

.markdown-preview :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}

.markdown-preview :deep(th),
.markdown-preview :deep(td) {
  border: 1px solid var(--border-color);
  padding: 6px 8px;
}

.form-textarea {
  resize: vertical;
  line-height: 1.6;
}

/* 图片展示区域 */
.images-gallery {
  margin-bottom: 20px;
}

.gallery-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.upload-section {
  margin-top: 16px;
  text-align: center;
}

.btn-add-images {
  width: 100%;
  padding: 12px 20px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.btn-add-images:hover {
  background: #1565c0;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(33, 150, 243, 0.3);
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
  margin-bottom: 16px;
}

.image-item {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition:
    transform 0.2s,
    box-shadow 0.2s;
  background: var(--bg-secondary);
  aspect-ratio: 3/2;
  border: 2px solid transparent;
}

.image-item:hover {
  transform: scale(1.03);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
  border-color: var(--primary-color);
}

/* 删除图片按钮 */
.delete-image-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  background: rgba(244, 67, 54, 0.9);
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 20px;
  font-weight: bold;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.2s;
  z-index: 10;
  padding: 0;
}

.image-item:hover .delete-image-btn {
  opacity: 1;
}

.delete-image-btn:hover {
  background: rgba(244, 67, 54, 1);
  transform: scale(1.1);
}

.question-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
  background: white;
}

.image-item::after {
  content: ' 点击预览';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 8px;
  text-align: center;
  font-size: 12px;
  opacity: 0;
  transition: opacity 0.2s;
}

.image-item:hover::after {
  opacity: 1;
}

/* 来源信息 */
.source-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--input-bg);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.source-display {
  font-size: 14px;
  color: var(--text-primary);
}

/* 标签区域 */
.tags-display {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.tag-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 16px;
  font-size: 13px;
  font-weight: 500;
}

.no-tags {
  color: var(--text-disabled);
  font-size: 13px;
  font-style: italic;
  padding: 8px 0;
  display: block;
}

.tags-edit {
  margin-top: 8px;
}

/* SRS 统计 */
.no-srs-data {
  text-align: center;
  padding: 24px;
}

.no-srs-data p {
  margin: 0;
  font-size: 13px;
  color: var(--text-disabled);
  font-style: italic;
}

.srs-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  background: var(--input-bg);
  border-radius: 8px;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

/* 时间信息 */
.time-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.time-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--input-bg);
  border-radius: 6px;
}

.time-label {
  font-size: 14px;
  color: var(--text-secondary);
}

.time-value {
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 500;
}

/* 小按钮 */
.btn-small {
  padding: 6px 12px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-small:hover {
  background: #1565c0;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 弹窗样式 */
.modal-overlay {
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

.source-modal-overlay {
  z-index: 1002;
}

.modal-content {
  background: var(--card-bg);
  border-radius: 12px;
  max-width: 800px;
  width: 95%;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.source-modal-content {
  min-height: 700px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

.modal-body {
  padding: 20px;
}
/*
.source-modal-body {
  min-height: 40px;
  padding: 24px;
} */

.confirm-modal .modal-body p {
  margin: 0;
  color: var(--text-primary);
  font-size: 15px;
  line-height: 1.6;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}

.btn-cancel,
.btn-confirm {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.btn-cancel:hover {
  background: var(--border-color);
}

.btn-confirm {
  background: #f44336;
  color: white;
}

.btn-confirm:hover {
  background: #d32f2f;
}

/* 保存按钮栏 */
.save-bar {
  position: fixed;
  bottom: 60px;
  left: 0;
  right: 0;
  background: rgba(255, 255, 255, 0.75);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  padding: 16px 20px;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: center;
  z-index: 1001;
  border-top: 1px solid rgba(255, 255, 255, 0.3);
}

/* 暗色主题适配 */
body.dark-theme .save-bar {
  background: rgba(47, 47, 47, 0.8);
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.save-btn {
  padding: 12px 32px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.save-btn:hover:not(:disabled) {
  background: #1565c0;
}

.save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .manage-detail-page {
    padding: 16px;
    padding-bottom: 100px;
  }

  .detail-header h2 {
    font-size: 18px;
  }

  .back-btn {
    padding: 6px 10px;
    font-size: 13px;
  }

  .back-icon {
    width: 14px;
    height: 14px;
  }

  .header-actions {
    gap: 8px;
  }

  .action-btn {
    padding: 6px 10px;
    font-size: 13px;
  }

  /* 移动端 - 双列布局（无表格线） */
  .srs-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px 16px;
    background: var(--input-bg);
    border-radius: 8px;
    padding: 12px;
  }

  .stat-item {
    display: contents;
  }

  .stat-label {
    padding: 8px 0;
    font-size: 12px;
    color: var(--text-secondary);
    text-align: left;
  }

  .stat-value {
    padding: 8px 0;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    text-align: right;
  }

  /* 移动端 - 删除弹窗适配 */
  .modal-overlay {
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-content.confirm-modal {
    width: 100%;
    max-width: unset;
    max-height: 90vh;
    overflow-y: auto;
    border-radius: 12px;
  }

  .confirm-modal .modal-header h3 {
    font-size: 16px;
  }

  .confirm-modal .modal-body p {
    font-size: 14px;
    line-height: 1.5;
  }

  .modal-footer {
    gap: 10px;
    padding: 12px 16px;
  }

  .btn-cancel,
  .btn-confirm {
    flex: 1;
    padding: 12px 16px;
    font-size: 14px;
  }
}

/* 桌面端优化 - 全屏显示 */
@media (min-width: 769px) {
  /* 图片网格优化 */
  .image-grid {
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 20px;
  }

  /* 表单元素优化 */
  .form-select,
  .form-textarea {
    font-size: 15px;
  }

  /* 弹窗宽度优化 */
  .modal-content {
    max-width: 900px;
  }
}
</style>

<!-- 全局覆盖 highlight.js 颜色（模板中无 .markdown-preview 类，所以 scoped 样式无效，必须用非 scoped） -->
<style>
.detail-content pre code.hljs {
  background: transparent !important;
  color: var(--code-text, #e2e8f0) !important;
}
.detail-content pre {
  background: var(--code-bg, #0f172a) !important;
}
</style>
