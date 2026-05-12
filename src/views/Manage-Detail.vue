<template>
  <div class="manage-detail-page">
    <!-- 顶部导航栏 -->
    <div class="detail-header">
      <button class="back-btn" @click="goBack">← 返回</button>
      <h2>错题详情管理</h2>
      <div class="header-actions">
        <button class="action-btn edit-btn" @click="toggleEditMode">
          {{ isEditing ? '取消编辑' : '编辑' }}
        </button>
        <button class="action-btn delete-btn" @click="confirmDelete">删除</button>
      </div>
    </div>

    <div v-if="errorDetail" class="detail-content">
      <!-- 基本信息区域 -->
      <div class="info-section">
        <div class="section-title">基本信息</div>
        
        <!-- 科目选择 -->
        <div class="form-group">
          <label>科目</label>
          <select 
            v-model="editForm.subject_id" 
            :disabled="!isEditing"
            class="form-select"
          >
            <option value="">请选择科目</option>
            <option v-for="subject in subjects" :key="subject.id" :value="subject.id">
              {{ subject.name }}
            </option>
          </select>
        </div>

        <!-- 来源信息 -->
        <div class="form-group">
          <label>来源信息</label>
          <div class="source-info">
            <div v-if="sourceInfo" class="source-display">
              <span v-if="sourceInfo.book">{{ sourceInfo.book }}</span>
              <span v-if="sourceInfo.chapter"> > {{ sourceInfo.chapter }}</span>
              <span v-if="sourceInfo.knowledge"> > {{ sourceInfo.knowledge }}</span>
              <span v-if="!sourceInfo.book && !sourceInfo.chapter && !sourceInfo.knowledge">未设置</span>
            </div>
            <button v-if="isEditing" class="btn-small" @click="openSourceSelector">
              修改来源
            </button>
          </div>
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
        <div v-if="(isEditing ? tempQuestionImages.length : questionImages.length) > 0" class="images-gallery">
          <div class="gallery-title">题目图片（点击预览）</div>
          <div class="image-grid">
            <div 
              v-for="(image, index) in (isEditing ? tempQuestionImages : questionImages)" 
              :key="image.id || `temp-${index}`" 
              class="image-item"
            >
              <img 
                :src="buildImageSrc(image)" 
                :alt="'题目图片'" 
                class="question-image"
                @click="previewImage(image)"
              >
              <button 
                v-if="isEditing"
                class="delete-image-btn"
                @click.stop="deleteTempImage(image)"
                title="删除图片"
              >
                ×
              </button>
            </div>
          </div>
        </div>
        
        <!-- 添加图片按钮（仅编辑模式） -->
        <div v-if="isEditing" class="upload-section">
          <button 
            class="btn-add-images"
            @click="triggerImageUpload"
          >
            📷 添加图片
          </button>
          <input 
            ref="imageInput" 
            type="file" 
            accept="image/*" 
            multiple
            style="display: none;"
            @change="handleImageSelect"
          />
        </div>
        
        <div class="form-group">
          <label>题干</label>
          <textarea 
            v-model="editForm.prompt" 
            :disabled="!isEditing"
            rows="6"
            class="form-textarea"
            placeholder="请输入题目内容..."
          ></textarea>
        </div>
      </div>

      <!-- 答案区域 -->
      <div class="answer-section">
        <div class="section-title">参考答案</div>
        
        <div class="form-group">
          <textarea 
            v-model="editForm.answer" 
            :disabled="!isEditing"
            rows="4"
            class="form-textarea"
            placeholder="请输入参考答案..."
          ></textarea>
        </div>
      </div>

      <!-- 解析区域 -->
      <div class="analysis-section">
        <div class="section-title">解析</div>
        
        <div class="form-group">
          <textarea 
            v-model="editForm.analysis" 
            :disabled="!isEditing"
            rows="6"
            class="form-textarea"
            placeholder="请输入题目解析..."
          ></textarea>
        </div>
      </div>

      <!-- 错因标签区域 -->
      <div class="tags-section">
        <div class="section-title">错因标签</div>
        
        <div class="tags-display">
          <span v-for="tag in errorTags" :key="tag.id" class="tag-item" :style="{ backgroundColor: tag.color + '20', color: tag.color }">
            {{ tag.name }}
          </span>
          <span v-if="errorTags.length === 0" class="no-tags">暂无标签</span>
        </div>
      </div>

      <!-- 错题笔记区域 -->
      <div class="note-section">
        <div class="section-title">错题笔记</div>
        
        <div class="form-group">
          <textarea 
            v-model="editForm.error_note" 
            :disabled="!isEditing"
            rows="6"
            class="form-textarea"
            placeholder="记录你的学习心得和注意事项..."
          ></textarea>
        </div>
      </div>

      <!-- SRS 数据展示 -->
      <div class="srs-section">
        <div class="section-title">学习数据</div>
        
        <div class="srs-stats">
          <div class="stat-item">
            <span class="stat-label">掌握程度</span>
            <span class="stat-value">{{ srsData?.mastery || 0 }}%</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">难度系数</span>
            <span class="stat-value">{{ srsData?.difficulty || '未设置' }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">复习次数</span>
            <span class="stat-value">{{ srsData?.review_count || 0 }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">最后复习</span>
            <span class="stat-value">{{ formatTimestamp(srsData?.lastreviewed_at) }}</span>
          </div>
        </div>
      </div>

      <!-- 时间信息 -->
      <div class="time-section">
        <div class="section-title">时间信息</div>
        <div class="time-info">
          <div class="time-item">
            <span class="time-label">创建时间：</span>
            <span class="time-value">{{ formatTimestamp(errorDetail.created_at) }}</span>
          </div>
          <div class="time-item">
            <span class="time-label">更新时间：</span>
            <span class="time-value">{{ formatTimestamp(errorDetail.updated_at) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-else class="loading-state">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 来源选择器弹窗 -->
    <div v-if="showSourceSelector" class="modal-overlay source-modal-overlay" @click="showSourceSelector = false">
      <div class="modal-content source-modal-content" @click.stop>
        <div class="modal-header">
          <h3>选择来源</h3>
          <button class="close-btn" @click="showSourceSelector = false">×</button>
        </div>
        <div class="modal-body source-modal-body">
          <SourceSelector 
            :currentSourceId="editForm.source_id || ''"
            :subjectId="editForm.subject_id"
            @select="handleSourceSelect"
          />
        </div>
      </div>
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
          <button class="btn-cancel" @click="showDeleteConfirm = false">取消</button>
          <button class="btn-confirm" @click="deleteError">确认删除</button>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div v-if="isEditing" class="save-bar">
      <button class="save-btn" @click="saveChanges" :disabled="saving">
        {{ saving ? '保存中...' : '保存修改' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { 
  getQuestion, 
  updateQuestion, 
  deleteQuestion 
} from '../apis/errorQuestions'
import { getSubjects } from '../apis/subjects'
import { getSource } from '../apis/sources'
import { getErrorTagByQuestionId, getErrorTags } from '../apis/errorTags'
import { getAttachmentsByQuestion, buildDataUrl, createAttachmentsForQuestion, fileToBase64, deleteAttachment, base64ToArrayBuffer } from '../apis/attachments'
import type { ErrorQuestion, Subject, Source, ErrorTags as ErrorTagType, Attachment } from '../types'
import SourceSelector from '../components/SourceSelector.vue'

const router = useRouter()
const route = useRoute()

// 错题ID
const errorId = computed(() => route.params.id as string)

// 数据状态
const errorDetail = ref<ErrorQuestion & { created_at?: number; updated_at?: number } | null>(null)
const subjects = ref<Subject[]>([])
const sourceInfo = ref<Source | null>(null)
const errorTags = ref<ErrorTagType[]>([])
const srsData = ref<any>(null)
const questionImages = ref<Attachment[]>([])
const answerImages = ref<Attachment[]>([])

// 图片上传相关
const imageInput = ref<HTMLInputElement | null>(null)
const uploading = ref(false)

// 临时图片列表（用于编辑时的暂存）
const tempQuestionImages = ref<Attachment[]>([])
const imagesToDelete = ref<string[]>([]) // 待删除的图片ID列表
const imagesToAdd = ref<File[]>([]) // 待添加的文件列表

// 编辑状态
const isEditing = ref(false)
const saving = ref(false)
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
const showSourceSelector = ref(false)
const showDeleteConfirm = ref(false)

// 获取错题详情
const fetchErrorDetail = async () => {
  try {
    const question = await getQuestion(errorId.value)
    
    // 处理后端返回的字段映射（subjectid -> subject_id, sourceid -> source_id 等）
    const mappedQuestion = {
      ...question,
      subject_id: (question as any).subjectid || question.subject_id,
      source_id: (question as any).sourceid || question.source_id,
      user_id: (question as any).userid || question.user_id,
      error_note: (question as any).error_note || '',
      created_at: (question as any).created_at,
      updated_at: (question as any).updated_at
    } as any
    
    errorDetail.value = mappedQuestion
    
    // 初始化编辑表单
    editForm.value = {
      subject_id: mappedQuestion.subject_id,
      source_id: mappedQuestion.source_id || '',
      prompt: mappedQuestion.prompt,
      type: (question as any).type_ || mappedQuestion.type,
      answer: mappedQuestion.answer || '',
      analysis: mappedQuestion.analysis || '',
      error_note: mappedQuestion.error_note || ''
    }
    
    // 获取来源信息
    if (mappedQuestion.source_id) {
      try {
        sourceInfo.value = await getSource(mappedQuestion.source_id)
      } catch (error) {
        console.error('获取来源信息失败:', error)
      }
    }
    
    // 获取标签
    try {
      const allTags = await getErrorTagByQuestionId(errorId.value)
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
      
      // 分类图片和答案图片（注意：后端字段名是 type_ 不是 type）
      questionImages.value = attachments.filter((att: any) => att.type_ === 'original')
      answerImages.value = attachments.filter((att: any) => att.type_ === 'answer')
      
      console.log('题目图片数量:', questionImages.value.length)
      console.log('答案图片数量:', answerImages.value.length)
    } catch (error) {
      console.error('获取题目图片失败:', error)
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
    // 清空临时图片数据
    tempQuestionImages.value = []
    imagesToDelete.value = []
    imagesToAdd.value = []
  } else {
    // 进入编辑模式，初始化临时列表
    tempQuestionImages.value = [...questionImages.value]
    imagesToDelete.value = []
    imagesToAdd.value = []
  }
  isEditing.value = !isEditing.value
}

// 保存修改
const saveChanges = async () => {
  if (!errorDetail.value) return
  
  saving.value = true
  try {
    // 1. 更新题目基本信息
    await updateQuestion({
      id: errorId.value,
      ...editForm.value
    })
    console.log('题目基本信息保存成功')
    
    // 2. 处理图片删除
    if (imagesToDelete.value.length > 0) {
      console.log('开始删除', imagesToDelete.value.length, '张图片')
      for (const imageId of imagesToDelete.value) {
        await deleteAttachment(imageId)
        console.log('已删除图片:', imageId)
      }
    }
    
    // 3. 处理图片添加
    if (imagesToAdd.value.length > 0) {
      console.log('开始上传', imagesToAdd.value.length, '张新图片')
      const attachments = []
      
      for (const file of imagesToAdd.value) {
        console.log('转换文件:', file.name)
        const base64Data = await fileToBase64(file)

        let fileType = 'png'
        if (file.type === 'image/jpeg' || file.type === 'image/jpg') {
          fileType = 'jpeg'
        } else if (file.type === 'image/webp') {
          fileType = 'webp'
        } else if (file.type === 'image/gif') {
          fileType = 'gif'
        }

        attachments.push({
          question_id: errorId.value,
          type_: 'original',
          file_type: fileType,
          base64_data: base64Data
        })
      }
      
      await createAttachmentsForQuestion(errorId.value, attachments)
      console.log('新图片上传成功')
    }
    
    // 4. 重新获取详情（包括最新的图片）
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
  }
}

// 确认删除
const confirmDelete = () => {
  showDeleteConfirm.value = true
}

// 删除错题
const deleteError = async () => {
  try {
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

// 打开来源选择器
const openSourceSelector = () => {
  showSourceSelector.value = true
}

// 处理来源选择
const handleSourceSelect = (sourceId: string) => {
  editForm.value.source_id = sourceId
  showSourceSelector.value = false
  // 重新获取来源信息
  if (sourceId) {
    getSource(sourceId).then(source => {
      sourceInfo.value = source
    }).catch(error => {
      console.error('获取来源信息失败:', error)
    })
  } else {
    sourceInfo.value = null
  }
}

// 构建图片src
const buildImageSrc = (attachment: any) => {
  console.log('构建图片URL:', attachment)
  try {
    // 如果是临时图片（包含原始文件引用）
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

// 预览图片
const previewImage = (attachment: any) => {
  console.log('预览图片:', attachment)
  const imageUrl = buildImageSrc(attachment)
  
  if (!imageUrl) {
    console.error('图片URL为空，无法预览')
    alert('图片加载失败')
    return
  }
  
  console.log('预览图片URL:', imageUrl.substring(0, 50) + '...')
  
  try {
    // 在新窗口打开图片
    const newWindow = window.open('', '_blank', 'width=1200,height=800')
    if (newWindow) {
      newWindow.document.write(`
        <!DOCTYPE html>
        <html>
        <head>
          <title>图片预览</title>
          <style>
            body {
              margin: 0;
              display: flex;
              justify-content: center;
              align-items: center;
              min-height: 100vh;
              background: #1a1a1a;
              overflow: auto;
            }
            img {
              max-width: 95%;
              max-height: 95vh;
              object-fit: contain;
              box-shadow: 0 4px 20px rgba(0,0,0,0.5);
            }
          </style>
        </head>
        <body>
          <img src="${imageUrl}" alt="图片预览">
        </body>
        </html>
      `)
      newWindow.document.close()
      console.log('预览窗口已打开')
    } else {
      console.error('无法打开预览窗口，可能被浏览器拦截')
      alert('无法打开预览窗口，请检查浏览器弹窗设置')
    }
  } catch (error) {
    console.error('预览图片失败:', error)
    alert('预览图片失败: ' + error)
  }
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
      const tempAttachment: any = {
        id: `temp-${Date.now()}-${i}`,
        question_id: errorId.value,
        type_: 'original',
        file_type: file.type.split('/')[1] || 'png',
        base64_data: '', // 稍后在保存时转换
        name: file.name,
        _file: file // 保存原始文件引用
      }
      tempQuestionImages.value.push(tempAttachment)
    }
    
    console.log('临时图片列表:', tempQuestionImages.value.length, '个')
    alert(`已添加 ${files.length} 张图片，请点击“保存修改”生效`)
  } catch (error) {
    console.error('处理图片失败:', error)
    alert('处理图片失败，请重试')
  } finally {
    // 清空 input，允许重复选择同一文件
    if (target) {
      target.value = ''
    }
  }
}

// 删除临时图片
const deleteTempImage = (image: any) => {
  if (!confirm('确定要删除这张图片吗？')) {
    return
  }
  
  console.log('删除临时图片:', image.id)
  
  // 如果是已有图片（有真实ID），加入待删除列表
  if (image.id && !image.id.startsWith('temp-')) {
    imagesToDelete.value.push(image.id)
    console.log('标记为待删除:', image.id)
  }
  
  // 从临时列表中移除
  tempQuestionImages.value = tempQuestionImages.value.filter(img => img.id !== image.id)
  console.log('删除成功，剩余图片:', tempQuestionImages.value.length, '个')
}

onMounted(() => {
  fetchErrorDetail()
  fetchSubjects()
})
</script>

<style scoped>
.manage-detail-page {
  padding: 20px;
  padding-bottom: 100px;
  background: var(--bg-primary);
  min-height: 100vh;
  max-width: 800px;
  margin: 0 auto;
}

/* 顶部导航 */
.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.back-btn {
  background: none;
  border: none;
  color: var(--primary-color);
  font-size: 16px;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 6px;
  transition: background 0.2s;
}

.back-btn:hover {
  background: var(--primary-light);
}

.detail-header h2 {
  font-size: 20px;
  margin: 0;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
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
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
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
  transition: transform 0.2s, box-shadow 0.2s;
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
  color: var(--text-secondary);
  font-size: 14px;
}

/* SRS 统计 */
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
  to { transform: rotate(360deg); }
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
  background: var(--card-bg);
  padding: 16px 20px;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: center;
  z-index: 1001;
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
  
  .detail-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
  
  .header-actions {
    width: 100%;
    justify-content: space-between;
  }
  
  .srs-stats {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
