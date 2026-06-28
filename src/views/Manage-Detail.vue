<template>
  <div class="manage-detail-page">
    <!-- 顶部导航栏 -->
    <div class="detail-header" ref="detailHeaderRef">
      <button class="back-btn" @click="goBack">
        <svg
          class="back-icon"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M19 12H5M12 19l-7-7 7-7" />
        </svg>
        <span>返回</span>
      </button>
      <h2>错题详情管理</h2>
      <div class="header-actions" ref="headerActionsRef" :class="{ collapsed: actionsCollapsed }">
        <button
          v-if="shareCheckDone && isShared && !isEditing"
          class="action-btn share-btn shared glare-btn"
          :disabled="shareLoading"
          @click="handleRevokeShare"
        >
          <svg class="btn-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/>
            <polyline points="16 6 12 2 8 6"/>
            <line x1="12" y1="2" x2="12" y2="15"/>
          </svg>
          <span class="btn-label">{{ shareLoading ? '...' : '撤回分享' }}</span>
        </button>
        <button
          v-if="shareCheckDone && !isShared && serverConfigured && !isEditing"
          class="action-btn share-btn glare-btn"
          :disabled="shareLoading"
          @click="handleShare"
        >
          <svg class="btn-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/>
            <polyline points="16 6 12 2 8 6"/>
            <line x1="12" y1="2" x2="12" y2="15"/>
          </svg>
          <span class="btn-label">{{ shareLoading ? '...' : '分享到社区' }}</span>
        </button>
        <button class="action-btn edit-btn glare-btn" @click="toggleEditMode">
          <svg class="btn-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
          <span class="btn-label">{{ isEditing ? '取消编辑' : '编辑' }}</span>
        </button>
        <button class="action-btn delete-btn glare-btn" @click="confirmDelete">
          <svg class="btn-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          <span class="btn-label">删除</span>
        </button>
      </div>
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
            :disable="!isEditing || sourceSelectorDisabled"
            :currentSourceId="editForm.source_id"
            :subjectId="editForm.subject_id"
            @select="handleSourceSelect"
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
          <button class="btn-cancel" @click="showDeleteConfirm = false">
            取消
          </button>
          <button class="btn-confirm" @click="deleteError">确认删除</button>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div v-if="isEditing" class="save-bar">
      <button class="save-btn glare-btn" @click="saveChanges" :disabled="saving">
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
  getQuestion,
  updateQuestion,
  deleteQuestion
} from '../apis/errorQuestions'
import { getSubjects } from '../apis/subjects'
import {
  getErrorTagByQuestionId,
  createErrorTagsForQuestion,
  deleteErrorTagById
} from '../apis/errorTags'
import {
  getAttachmentsByQuestion,
  buildDataUrl,
  createAttachmentsForQuestion,
  fileToBase64,
  deleteAttachment
} from '../apis/attachments'
import { getQuestionSRSStatus } from '../apis/srsData'
import { publishShare, revokeShare, checkShare } from '../apis/share'
import type {
  ErrorQuestion,
  Subject,
  ErrorTags as ErrorTagType,
  Attachment
} from '../types'
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
const fullButtonsWidth = ref(0)    // 挂载时测量的按钮全宽（含文字）
const backBtnWidth = ref(0)        // 挂载时测量的返回按钮宽度
const titleMinWidth = ref(0)       // 挂载时测量的标题最小宽度
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

// 分享状态
const shareLoading = ref(false)
const shareCheckDone = ref(false)
const isShared = ref(false)
const serverConfigured = ref(false)

/**
 * 检查服务器是否已配置
 */
function checkServerConfig(): boolean {
  const url = localStorage.getItem('sync_server_url')
  const auth = localStorage.getItem('auth_key')
  return !!(url && auth)
}

/**
 * 检查当前题目是否已分享
 */
async function checkShareStatus() {
  if (!serverConfigured.value) return

  const authKey = localStorage.getItem('auth_key') || ''
  try {
    isShared.value = await checkShare({ auth_key: authKey, id: errorId.value })
  } catch {
    isShared.value = false
  } finally {
    shareCheckDone.value = true
  }
}

/**
 * 分享当前错题到社区
 */
async function handleShare() {
  if (!errorDetail.value || shareLoading.value) return

  shareLoading.value = true
  const authKey = localStorage.getItem('auth_key') || ''
  const detail = errorDetail.value as any

  try {
    await publishShare({
      auth_key: authKey,
      id: errorId.value,
      prompt: detail.prompt,
      type_: detail.type_ || detail.type || '',
      answer: detail.answer || '',
      analysis: detail.analysis || '',
      error_note: detail.error_note || '',
    })
    isShared.value = true
  } catch (e: any) {
    alert('分享失败: ' + (e.message || '未知错误'))
  } finally {
    shareLoading.value = false
  }
}

/**
 * 撤回分享
 */
async function handleRevokeShare() {
  if (shareLoading.value) return

  shareLoading.value = true
  const authKey = localStorage.getItem('auth_key') || ''

  try {
    await revokeShare({ auth_key: authKey, id: errorId.value })
    isShared.value = false
  } catch (e: any) {
    alert('撤回失败: ' + (e.message || '未知错误'))
  } finally {
    shareLoading.value = false
  }
}

// 获取错题详情
const fetchErrorDetail = async () => {
  console.log('========== 开始获取错题详情 ==========')
  console.log('错题ID:', errorId.value)

  try {
    const question = await getQuestion(errorId.value)
    console.log('========== 后端返回的原始数据 ==========')
    console.log('question 对象:', question)
    console.log('question 类型:', typeof question)
    console.log('question.keys:', Object.keys(question))

    // 详细记录所有字段
    console.log('========== 字段详细检查 ==========')
    console.log('question.id:', (question as any).id)
    console.log('question.subjectid:', (question as any).subjectid)
    console.log('question.subject_id:', (question as any).subject_id)
    console.log('question.sourceid:', (question as any).sourceid)
    console.log('question.source_id:', (question as any).source_id)
    console.log('question.userid:', (question as any).userid)
    console.log('question.user_id:', (question as any).user_id)
    console.log('question.prompt:', (question as any).prompt)
    console.log('question.type_:', (question as any).type_)
    console.log('question.type:', (question as any).type)
    console.log('question.answer:', (question as any).answer)
    console.log('question.analysis:', (question as any).analysis)
    console.log('question.error_note:', (question as any).error_note)

    // 处理后端返回的字段映射（subjectid -> subject_id, sourceid -> source_id 等）
    // 注意：使用 ?? 运算符以正确保留 null 值
    const mappedQuestion = {
      ...question,
      subject_id: (question as any).subjectid ?? question.subject_id,
      source_id: (question as any).sourceid ?? question.source_id,
      user_id: (question as any).userid ?? question.user_id,
      error_note: (question as any).error_note || '',
      created_at: (question as any).created_at,
      updated_at: (question as any).updated_at
    } as any

    console.log('========== 映射后的数据 ==========')
    console.log('mappedQuestion:', mappedQuestion)
    console.log('mappedQuestion.subject_id:', mappedQuestion.subject_id)
    console.log('mappedQuestion.source_id:', mappedQuestion.source_id)
    console.log(
      'mappedQuestion.source_id 类型:',
      typeof mappedQuestion.source_id
    )
    console.log('mappedQuestion.source_id 是否为空:', !mappedQuestion.source_id)

    errorDetail.value = mappedQuestion
    console.log('errorDetail.value 已设置')

    // 初始化编辑表单
    // 注意：source_id 使用 ?? 保留 null 值，避免 || 运算符将 null 转换为 undefined
    editForm.value = {
      subject_id: mappedQuestion.subject_id,
      source_id: mappedQuestion.source_id ?? '',
      prompt: mappedQuestion.prompt,
      type: (question as any).type_ || mappedQuestion.type,
      answer: mappedQuestion.answer || '',
      analysis: mappedQuestion.analysis || '',
      error_note: mappedQuestion.error_note || ''
    }

    console.log('========== 编辑表单初始化 ==========')
    console.log('editForm.value:', editForm.value)
    console.log('editForm.value.subject_id:', editForm.value.subject_id)
    console.log('editForm.value.source_id:', editForm.value.source_id)
    console.log(
      'editForm.value.source_id 类型:',
      typeof editForm.value.source_id
    )
    console.log('editForm.value.source_id 是否为空:', !editForm.value.source_id)

    // 获取标签
    try {
      console.log('开始获取标签...')
      const allTags = await getErrorTagByQuestionId(errorId.value)
      console.log('获取到的标签数量:', allTags.length)
      errorTags.value = allTags
    } catch (error) {
      console.error('获取标签失败:', error)
    }

    // 获取题目图片
    try {
      console.log('开始获取题目附件...')
      console.log('错题ID:', errorId.value)

      if (!errorId.value) {
        console.error('错题ID为空，无法获取附件')
        return
      }

      const attachments = await getAttachmentsByQuestion(errorId.value)
      console.log('获取到的附件:', attachments)

      // 分类附件（注意：后端字段名是 type_ 不是 type）
      questionImages.value = attachments.filter(
        (a: any) => a.type_ === 'original'
      )
      answerImages.value = attachments.filter((a: any) => a.type_ === 'answer')
      console.log('题目图片数量:', questionImages.value.length)
      console.log('答案图片数量:', answerImages.value.length)
    } catch (error) {
      console.error('获取附件失败:', error)
    }

    // 获取 SRS 数据
    try {
      console.log('开始获取 SRS 数据...')
      const srs = await getQuestionSRSStatus(errorId.value)
      if (srs) {
        srsData.value = srs
        console.log('SRS 数据:', srs)
      } else {
        console.log('该题目没有 SRS 数据')
        srsData.value = null
      }
    } catch (error) {
      console.error('获取 SRS 数据失败:', error)
      srsData.value = null
    }
  } catch (error) {
    console.error('获取错题详情失败:', error)
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
  if (!errorDetail.value) return

  console.log('========== 开始保存流程 ==========')
  console.log('1. 先禁用SourceSelector')

  // 1. 先禁用SourceSelector，确保下拉框收起
  sourceSelectorDisabled.value = true

  // 2. 等待一小段时间，确保组件响应disabled状态并收起
  await new Promise((resolve) => setTimeout(resolve, 100))

  console.log('2. SourceSelector已禁用，开始执行保存操作')

  saving.value = true
  try {
    // 1. 更新题目基本信息
    console.log('========== 开始保存题目基本信息 ==========')
    console.log('editForm:', JSON.parse(JSON.stringify(editForm.value)))
    console.log('editForm.source_id:', editForm.value.source_id)
    console.log('editForm.subject_id:', editForm.value.subject_id)

    const updateData = {
      id: errorId.value,
      ...editForm.value
    }
    console.log(
      '准备发送给后端的 updateData:',
      JSON.parse(JSON.stringify(updateData))
    )

    await updateQuestion(updateData)
    console.log('题目基本信息保存成功')

    // 3. 处理图片更新（包括新增、删除和编辑）
    console.log('========== 开始处理图片 ==========')
    console.log('原始图片数量:', questionImages.value.length)
    console.log('临时图片数量:', tempQuestionImages.value.length)
    console.log('待删除图片ID:', imagesToDelete.value)

    // 3.1 找出被编辑过的图片（在临时列表中存在但 base64_data 已改变的图片）
    const editedImageIds: string[] = []
    console.log('开始检测编辑过的图片...')
    for (const tempImg of tempQuestionImages.value) {
      console.log('\n--- 检查图片 ---')
      console.log('图片ID:', tempImg.id)
      console.log('是否为临时图片:', tempImg.id.startsWith('temp-'))

      // 跳过新添加的临时图片（ID以temp-开头）
      if (tempImg.id.startsWith('temp-')) {
        console.log('跳过新添加的临时图片')
        continue
      }

      // 查找原始图片列表中对应的图片
      const originalImg = questionImages.value.find(
        (img) => img.id === tempImg.id
      )
      console.log('找到原始图片:', !!originalImg)

      if (originalImg) {
        const originalLen = originalImg.base64_data?.length || 0
        const tempLen = tempImg.base64_data?.length || 0
        const isSame = originalImg.base64_data === tempImg.base64_data

        console.log('原始 base64 长度:', originalLen)
        console.log('临时 base64 长度:', tempLen)
        console.log('长度差异:', Math.abs(originalLen - tempLen))
        console.log('base64 是否相同:', isSame)

        if (!isSame) {
          console.log('✅ 发现编辑过的图片:', tempImg.id)
          console.log(
            '   原始前50字符:',
            originalImg.base64_data?.substring(0, 50)
          )
          console.log('   临时前50字符:', tempImg.base64_data?.substring(0, 50))
          editedImageIds.push(tempImg.id)
        } else {
          console.log('❌ 图片未编辑')
        }
      } else {
        console.log('⚠️ 未找到原始图片')
      }
    }
    console.log('\n========== 编辑过的图片ID列表 ==========')
    console.log(editedImageIds)

    // 3.2 删除所有需要删除或更新的图片（包括显式删除的和编辑过的）
    const allImagesToDelete = [...imagesToDelete.value, ...editedImageIds]
    if (allImagesToDelete.length > 0) {
      console.log('开始删除', allImagesToDelete.length, '张图片')
      for (const imageId of allImagesToDelete) {
        await deleteAttachment(imageId)
        console.log('已删除图片:', imageId)
      }
    }

    // 3.3 上传需要新增或更新的图片（只上传编辑后的和新添加的）
    const imagesToUpload: any[] = []

    // 添加编辑后的图片（使用新的 base64 数据）
    console.log('\n========== 准备上传编辑后的图片 ==========')
    for (const tempImg of tempQuestionImages.value) {
      if (editedImageIds.includes(tempImg.id)) {
        console.log('准备上传编辑后的图片:')
        console.log('  ID:', tempImg.id)
        console.log('  type_:', (tempImg as any).type_ || tempImg.type)
        console.log('  file_type:', tempImg.file_type)
        console.log('  base64_data 长度:', tempImg.base64_data?.length || 0)

        imagesToUpload.push({
          question_id: errorId.value,
          type_: (tempImg as any).type_ || tempImg.type,
          file_type: tempImg.file_type,
          base64_data: tempImg.base64_data
        })
      }
    }

    // 添加新上传的图片（优先使用 tempQuestionImages 中已编辑的数据）
    console.log('\n========== 准备上传新添加的图片 ==========')
    console.log('imagesToAdd 数量:', imagesToAdd.value.length)
    console.log('tempQuestionImages 数量:', tempQuestionImages.value.length)

    if (imagesToAdd.value.length > 0) {
      for (let i = 0; i < imagesToAdd.value.length; i++) {
        const file = imagesToAdd.value[i]
        console.log(`\n处理新图片 ${i + 1}:`, file.name)

        // 查找对应的临时图片对象（可能已经被编辑过）
        const tempImg = tempQuestionImages.value.find((img) => {
          if (!img.id.startsWith('temp-')) return false
          const imgAny = img as any
          return imgAny._file === file
        })

        if (tempImg && tempImg.base64_data && tempImg.base64_data.length > 0) {
          // 如果临时图片已经有 base64_data（可能被编辑过），直接使用
          console.log('✅ 使用已编辑的 base64 数据')
          console.log('  base64_data 长度:', tempImg.base64_data.length)

          imagesToUpload.push({
            question_id: errorId.value,
            type_: 'original',
            file_type: tempImg.file_type,
            base64_data: tempImg.base64_data
          })
        } else {
          // 否则从原始文件转换
          console.log('⚠️ 从原始文件转换 base64')
          const base64Data = await fileToBase64(file)

          let fileType = 'png'
          if (file.type === 'image/jpeg' || file.type === 'image/jpg') {
            fileType = 'jpeg'
          } else if (file.type === 'image/webp') {
            fileType = 'webp'
          } else if (file.type === 'image/gif') {
            fileType = 'gif'
          }

          imagesToUpload.push({
            question_id: errorId.value,
            type_: 'original',
            file_type: fileType,
            base64_data: base64Data
          })
        }
      }
    }

    // 批量创建所有需要保留的图片
    if (imagesToUpload.length > 0) {
      console.log('开始上传', imagesToUpload.length, '张图片')
      await createAttachmentsForQuestion(errorId.value, imagesToUpload)
      console.log('图片上传成功')
    }

    // 4. 处理标签更新
    console.log('========== 开始更新标签 ==========')
    console.log('旧标签数量:', errorTags.value.length)
    console.log(
      '旧标签列表:',
      errorTags.value.map((t) => `${t.name}(${t.id})`)
    )
    console.log('新标签数量:', tempErrorTags.value.length)
    console.log(
      '新标签列表:',
      tempErrorTags.value.map((t) => `${t.name}(${t.color})`)
    )

    // 4.1 删除当前题目的所有旧标签（使用ID精确删除）
    if (errorTags.value.length > 0) {
      console.log('开始删除', errorTags.value.length, '个旧标签...')
      for (const tag of errorTags.value) {
        try {
          console.log('删除标签:', tag.name, '(ID:', tag.id, ')')
          const deletedCount = await deleteErrorTagById(tag.id)
          console.log('已删除标签:', tag.name, '(影响行数:', deletedCount, ')')
        } catch (error) {
          console.error('删除标签失败:', tag.name, error)
        }
      }
      console.log('所有旧标签删除完成')
    } else {
      console.log('没有旧标签需要删除')
    }

    // 4.2 创建新标签（只为当前题目创建）
    if (tempErrorTags.value.length > 0) {
      console.log('开始创建', tempErrorTags.value.length, '个新标签...')
      const createdTags = await createErrorTagsForQuestion(
        errorId.value,
        tempErrorTags.value
      )
      console.log('新标签创建成功，创建了', createdTags.length, '个标签')
      console.log('创建的标签:', createdTags)
    } else {
      console.log('没有新标签需要创建')
    }

    console.log('========== 标签更新完成 ==========')

    // 5. 重新获取详情（包括最新的图片和标签）
    await fetchErrorDetail()
    isEditing.value = false

    // 清空临时数据
    tempQuestionImages.value = []
    imagesToDelete.value = []
    imagesToAdd.value = []

    // 显示成功提示
    alert('保存成功！')
  } catch (error) {
    console.error('保存失败:', error)
    alert('保存失败，请重试')
  } finally {
    saving.value = false
    // 恢复SourceSelector的disabled状态（根据isEditing决定）
    sourceSelectorDisabled.value = false
    console.log('3. 保存完成，恢复SourceSelector状态')
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
    // 如果已分享，异步撤回（不阻塞主操作）
    if (isShared.value && serverConfigured.value) {
      const authKey = localStorage.getItem('auth_key') || ''
      revokeShare({ auth_key: authKey, id: errorId.value }).catch(() => {
        // 忽略撤回失败，删除是主要操作
      })
    }
    await deleteQuestion(errorId.value)
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
  console.log('========== 科目选择事件 ==========')
  console.log('选中的科目ID:', subjectId)
  editForm.value.subject_id = subjectId
  console.log('editForm.value.subject_id:', editForm.value.subject_id)
}

// 处理来源选择
const handleSourceSelect = (sourceId: string) => {
  console.log('========== 来源选择事件 ==========')
  console.log('选中的来源ID:', sourceId)
  console.log('选中前 editForm.source_id:', editForm.value.source_id)
  editForm.value.source_id = sourceId
  console.log('选中后 editForm.source_id:', editForm.value.source_id)

  // 验证是否真的更新了
  setTimeout(() => {
    console.log('100ms 后 editForm.source_id:', editForm.value.source_id)
  }, 100)
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
  serverConfigured.value = checkServerConfig()
  fetchSubjects()

  // 按钮自适应：等数据加载完成（分享按钮 v-if 依赖 shareCheckDone），再测量+启动观察
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
        const totalNeeded = backBtnWidth.value + fullButtonsWidth.value + titleMinWidth.value + 16

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
        const initNeeded = backBtnWidth.value + fullButtonsWidth.value + titleMinWidth.value + 16
        if (initWidth < initNeeded - 2) {
          actionsCollapsed.value = true
        }
      }
    })
  }

  // 等异步数据全部到位后再测量
  fetchErrorDetail().then(() => {
    if (serverConfigured.value) {
      checkShareStatus().finally(() => {
        setupAdaptive()
      })
    } else {
      setupAdaptive()
    }
  })
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

.share-btn {
  background: var(--success-color);
  color: white;
}

.share-btn:hover {
  background: var(--success-color);
  filter: brightness(0.85);
}

.share-btn.shared {
  background: var(--warning-color);
}

.share-btn.shared:hover {
  background: var(--warning-color);
  filter: brightness(0.85);
}

.share-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04);
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
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08), 0 2px 8px rgba(0, 0, 0, 0.06);
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
  background: #0f172a;
  padding: 10px;
  border-radius: 6px;
  overflow-x: auto;
}

.markdown-preview :deep(pre code) {
  background: transparent;
  padding: 0;
  color: #e2e8f0;
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
