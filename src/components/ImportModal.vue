<template>
  <div class="import-modal-overlay" @click.self="handleClose">
    <div class="import-modal">
      <!-- 标题 -->
      <div class="modal-header">
        <h2 class="modal-title">
          {{
            step === 'select'
              ? '导入错题'
              : step === 'review'
                ? `导入题目 (${currentIndex + 1}/${totalCount})`
                : '导入结果'
          }}
        </h2>
        <button class="modal-close-btn" @click="handleClose">
          <Icon name="x" :size="18" />
        </button>
      </div>

      <div class="modal-body">
        <!-- ===== 步骤 1：选择文件 ===== -->
        <div v-if="step === 'select'" class="step-container">
          <div class="step-content">
            <div class="file-select-area" @click="handleSelectFile">
              <Icon name="file-text" :size="48" />
              <div class="file-select-text">
                <div class="file-select-title">点击选择 JSON 文件</div>
                <div class="file-select-desc">
                  支持 SmartErrorNotebook 导出的 .json 格式
                </div>
              </div>
            </div>
            <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
          </div>
        </div>

        <!-- ===== 步骤 2：逐题审查 ===== -->
        <div v-else-if="step === 'review'" class="step-container">
          <!-- 进度条 -->
          <div class="progress-bar">
            <div
              class="progress-fill"
              :style="{ width: progressPercent + '%' }"
            ></div>
          </div>

          <!-- 版本警告 -->
          <div v-if="versionWarning" class="version-warning">
            <Icon name="triangle-alert" :size="14" />
            文件版本 "{{ versionWarning }}" 与当前版本不匹配
          </div>

          <!-- 题目卡片 -->
          <div class="import-review-card" v-if="currentQuestion">
            <!-- 题干 -->
            <div class="review-section">
              <div class="review-label">题目</div>
              <div
                class="review-content markdown-body"
                v-html="currentQuestion.promptHtml"
              ></div>
            </div>

            <!-- 答案（可折叠） -->
            <div class="review-section" v-if="currentQuestion.answer">
              <div class="review-label collapsible" @click="toggleAnswer">
                参考答案
                <span class="collapse-icon">{{ showAnswer ? '▲' : '▼' }}</span>
              </div>
              <div
                v-show="showAnswer"
                class="review-content markdown-body"
                v-html="currentQuestion.answerHtml"
              ></div>
            </div>

            <!-- 解析（可折叠） -->
            <div class="review-section" v-if="currentQuestion.analysis">
              <div class="review-label collapsible" @click="toggleAnalysis">
                解析
                <span class="collapse-icon">{{
                  showAnalysis ? '▲' : '▼'
                }}</span>
              </div>
              <div
                v-show="showAnalysis"
                class="review-content markdown-body"
                v-html="currentQuestion.analysisHtml"
              ></div>
            </div>

            <!-- 配置区 -->
            <div class="review-config">
              <div class="config-row">
                <div class="config-field">
                  <label>科目 <span class="required">*</span></label>
                  <SubjectSelector
                    :model-value="reviewSubjectId"
                    @select="setReviewSubject"
                  />
                </div>
                <div class="config-field">
                  <label>题型</label>
                  <select v-model="reviewType" class="type-select">
                    <option
                      v-for="t in questionTypes"
                      :key="t.value"
                      :value="t.value"
                    >
                      {{ t.label }}
                    </option>
                  </select>
                </div>
              </div>
              <!-- 错因标签 -->
              <div class="config-field" style="margin-top: 8px">
                <label>错因标签</label>
                <ErrorTagSelector
                  :current-tags="reviewTags"
                  @select="setReviewTags"
                />
              </div>
              <!-- 应用到全部 -->
              <label class="apply-all-check" v-if="currentIndex > 0">
                <input type="checkbox" v-model="applyToAll" />
                <span>将此配置应用到剩余题目</span>
              </label>
            </div>

            <!-- 去重提示 -->
            <div v-if="isDuplicate" class="duplicate-tag">
              <Icon name="info" :size="14" />
              此题目与已有题目重复（prompt 匹配），导入将跳过
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="review-actions">
            <span class="counter"
              >第 {{ currentIndex + 1 }} / {{ totalCount }} 题</span
            >
            <div class="action-group">
              <button
                class="act-btn skip-btn"
                @click="skipQuestion"
                :disabled="saving"
              >
                跳过
              </button>
              <button
                v-if="isDuplicate"
                class="act-btn skip-btn"
                @click="skipQuestion"
                :disabled="saving"
              >
                跳过重复
              </button>
              <button
                v-else
                class="act-btn import-btn"
                @click="importCurrent"
                :disabled="!reviewSubjectId || saving"
              >
                {{ saving ? '导入中...' : '导入此题目' }}
              </button>
              <button
                class="act-btn import-all-btn"
                @click="importAllRemaining"
                :disabled="!reviewSubjectId || saving"
              >
                导入剩余全部
              </button>
            </div>
          </div>
        </div>

        <!-- ===== 步骤 3：结果 ===== -->
        <div
          v-else-if="step === 'result'"
          class="step-container result-container"
        >
          <div class="result-icon" :class="resultClass">
            {{ resultIcon }}
          </div>
          <div class="result-title">{{ resultTitle }}</div>
          <div class="result-detail">{{ resultTitle }}</div>
          <div v-if="resultErrors.length" class="result-errors">
            <div
              v-for="(e, i) in resultErrors"
              :key="i"
              class="result-error-item"
            >
              {{ e }}
            </div>
          </div>
        </div>
      </div>

      <!-- 底部 -->
      <div class="modal-footer">
        <button class="cancel-btn" @click="handleClose">
          {{ step === 'result' ? '关闭' : '取消' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import Icon from './Icon.vue'
import SubjectSelector from './SubjectSelector.vue'
import ErrorTagSelector from './ErrorTagSelector.vue'
import { QuestionType } from '../types'
import {
  parseImportFile,
  importSingleQuestion,
  getExistingPromptSet
} from '../utils/importJson'
import { Marked } from 'marked'
import markedKatex from 'marked-katex-extension'

// ===== marked 渲染 =====
const _marked = new Marked(
  markedKatex({
    throwOnError: false,
    output: 'html',
    nonStandard: true,
    strict: 'ignore'
  }),
  {
    renderer: {
      code({ text, lang }) {
        const e = text
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
        return `<pre><code class="hljs ${lang ? `language-${lang}` : ''}">${e}</code></pre>`
      }
    }
  }
)

function renderMd(t: string | undefined | null): string {
  if (!t) return ''
  const c = (t || '').replace(/[①-⑳]/g, (m) => `(${m.charCodeAt(0) - 0x245f})`)
  return _marked.parse(
    c
      .replace(/\\\[/g, '$$$$')
      .replace(/\\\]/g, '$$$$')
      .replace(/\\\(/g, '$')
      .replace(/\\\)/g, '$'),
    { breaks: true, gfm: true }
  ) as string
}

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'import-complete'): void
}>()

const props = defineProps<{
  /** 从外部传入的已解析数据，跳过「选择文件」步骤直接进入审查 */
  initialData?: {
    questions: Array<{ prompt: string; answer?: string; analysis?: string }>
    version?: string
  } | null
}>()

// ===== 状态 =====
const step = ref<'select' | 'review' | 'result'>('select')
const errorMsg = ref('')
const versionWarning = ref('')
const questions = ref<
  {
    prompt: string
    answer: string
    analysis: string
    promptHtml: string
    answerHtml: string
    analysisHtml: string
  }[]
>([])
const currentIndex = ref(0)
const saving = ref(false)
const existingPrompts = ref<Set<string>>(new Set())
// 累积统计
const accumSuccess = ref(0)
const accumSkipped = ref(0)
const accumFailed = ref(0)
const accumErrors = ref<string[]>([])

// 显示折叠
const showAnswer = ref(false)
const showAnalysis = ref(false)
const toggleAnswer = () => {
  showAnswer.value = !showAnswer.value
}
const toggleAnalysis = () => {
  showAnalysis.value = !showAnalysis.value
}

// 配置
const reviewSubjectId = ref('')
const reviewType = ref(QuestionType.ShortAnswer)
const reviewTags = ref<Array<{ name: string; color: string }>>([])
const applyToAll = ref(false)
const questionTypes = Object.entries(QuestionType).map(([v, l]) => ({
  value: v,
  label: l
}))

const totalCount = computed(() => questions.value.length)
const progressPercent = computed(() =>
  totalCount.value ? (currentIndex.value / totalCount.value) * 100 : 0
)
const currentQuestion = computed(
  () => questions.value[currentIndex.value] || null
)
const isDuplicate = computed(() =>
  currentQuestion.value
    ? existingPrompts.value.has(currentQuestion.value.prompt.trim())
    : false
)

function setReviewSubject(id: string) {
  reviewSubjectId.value = id
}
function setReviewTags(tags: Array<{ name: string; color: string }>) {
  reviewTags.value = tags
}

// ===== 从解析后的数据进入审查模式（抽取为公共方法） =====
const enterReviewMode = async (
  result: ReturnType<typeof parseImportFile>,
  existingPromptsInput?: Set<string>
) => {
  if (result.error) {
    errorMsg.value = result.error
    return
  }

  // 版本警告
  if (result.version && result.version !== '1.0') {
    versionWarning.value = result.version
  }

  // 构建带 HTML 渲染的题目列表
  questions.value = result.questions.map((q) => ({
    prompt: q.prompt,
    answer: q.answer || '',
    analysis: q.analysis || '',
    promptHtml: renderMd(q.prompt),
    answerHtml: renderMd(q.answer),
    analysisHtml: renderMd(q.analysis)
  }))

  // 获取已有题目用于去重
  existingPrompts.value =
    existingPromptsInput || (await getExistingPromptSet())

  // 进入审查
  currentIndex.value = 0
  reviewSubjectId.value = ''
  reviewType.value = QuestionType.ShortAnswer
  applyToAll.value = false
  showAnswer.value = false
  showAnalysis.value = false
  step.value = 'review'
}

// ===== 选择文件 =====
const handleSelectFile = async () => {
  try {
    errorMsg.value = ''
    const { open } = await import('@tauri-apps/plugin-dialog')
    const { readTextFile } = await import('@tauri-apps/plugin-fs')
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'JSON 文件', extensions: ['json'] }]
    })
    if (!filePath) return

    const content = await readTextFile(filePath as string)
    const result = parseImportFile(content)

    await enterReviewMode(result)
  } catch (e) {
    errorMsg.value = String(e)
  }
}

// ===== 如果外部传入 initialData，直接跳过文件选择进入审查 =====
onMounted(() => {
  if (props.initialData?.questions && props.initialData.questions.length > 0) {
    const data = props.initialData
    const result = {
      questions: data.questions.map((q) => ({
        prompt: q.prompt,
        answer: q.answer || '',
        analysis: q.analysis || ''
      })),
      version: data.version || '',
      error: undefined as string | undefined
    }
    enterReviewMode(result)
  }
})

// ===== 跳过 =====
const skipQuestion = () => {
  advanceToNext()
}

// ===== 导入当前题 =====
const importCurrent = async () => {
  if (!reviewSubjectId.value || !currentQuestion.value || saving.value) return
  saving.value = true
  try {
    if (isDuplicate.value) {
      accumSkipped.value++
      advanceToNext()
      return
    }
    const result = await importSingleQuestion(
      currentQuestion.value,
      reviewSubjectId.value,
      reviewType.value,
      'default-user',
      reviewTags.value.length > 0 ? reviewTags.value : undefined
    )
    if (result.success) {
      accumSuccess.value++
      existingPrompts.value.add(currentQuestion.value.prompt.trim())
    } else {
      accumFailed.value++
      accumErrors.value.push(
        `第 ${currentIndex.value + 1} 题: ${result.error || ''}`
      )
    }
  } catch (e) {
    accumFailed.value++
    accumErrors.value.push(`第 ${currentIndex.value + 1} 题: ${String(e)}`)
  } finally {
    saving.value = false
    advanceToNext()
  }
}

// ===== 导入剩余全部 =====
const importAllRemaining = async () => {
  if (!reviewSubjectId.value || saving.value) return
  saving.value = true

  for (let i = currentIndex.value; i < questions.value.length; i++) {
    const q = questions.value[i]
    if (existingPrompts.value.has(q.prompt.trim())) {
      accumSkipped.value++
      continue
    }
    try {
      await importSingleQuestion(
        q,
        reviewSubjectId.value,
        reviewType.value,
        'default-user',
        reviewTags.value.length > 0 ? reviewTags.value : undefined
      )
      existingPrompts.value.add(q.prompt.trim())
      accumSuccess.value++
    } catch (e) {
      accumFailed.value++
      accumErrors.value.push(`第 ${i + 1} 题: ${String(e)}`)
    }
  }

  saving.value = false
  showFinalResult()
}

// ===== 下一题 =====
const advanceToNext = () => {
  if (currentIndex.value >= questions.value.length - 1) {
    showFinalResult()
    return
  }
  currentIndex.value++
  showAnswer.value = false
  showAnalysis.value = false
  if (!applyToAll.value) {
    reviewSubjectId.value = ''
  }
}

// ===== 显示最终结果 =====
const resultStats = ref({
  success: 0,
  skipped: 0,
  failed: 0,
  errors: [] as string[]
})

const resultClass = computed(() => {
  const s = resultStats.value
  if (s.failed > 0) return 'warning'
  if (s.success > 0) return 'success'
  return 'info'
})

const resultIcon = computed(() => {
  const s = resultStats.value
  if (s.failed > 0) return '⚠️'
  if (s.success > 0) return '✅'
  return 'ℹ️'
})

const resultTitle = computed(() => {
  const s = resultStats.value
  const parts: string[] = []
  if (s.success > 0) parts.push(`成功 ${s.success} 题`)
  if (s.skipped > 0) parts.push(`跳过 ${s.skipped} 题`)
  if (s.failed > 0) parts.push(`失败 ${s.failed} 题`)
  return parts.join('，') || '导入完成'
})

const resultErrors = computed(() => resultStats.value.errors)

const showFinalResult = () => {
  resultStats.value = {
    success: accumSuccess.value,
    skipped: accumSkipped.value,
    failed: accumFailed.value,
    errors: accumErrors.value
  }
  step.value = 'result'
  emit('import-complete')
}

const handleClose = () => {
  if (step.value === 'result') emit('close')
  else emit('close')
}
</script>

<style scoped>
.import-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: fadeIn 0.2s ease-out;
}

.import-modal {
  background: var(--card-bg);
  border-radius: var(--radius-lg);
  width: 560px;
  max-width: 92vw;
  max-height: 88vh;
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  animation: slideUp 0.25s ease-out;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.modal-close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}
.modal-close-btn:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
}

.cancel-btn {
  padding: 8px 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}
.cancel-btn:hover {
  background: var(--bg-tertiary);
}

/* ===== 步骤容器 ===== */
.step-container {
  animation: fadeIn 0.2s ease-out;
}

/* ===== 文件选择 ===== */
.file-select-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px 20px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-secondary);
}
.file-select-area:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
  background: var(--primary-light);
}
.file-select-title {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 4px;
  text-align: center;
}
.file-select-desc {
  font-size: 13px;
  color: var(--text-hint);
}

.error-msg {
  margin-top: 12px;
  padding: 10px 14px;
  background: var(--danger-light);
  color: var(--danger-color);
  border-radius: var(--radius-md);
  font-size: 13px;
  white-space: pre-wrap;
}

/* ===== 进度条 ===== */
.progress-bar {
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  margin-bottom: 12px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: var(--primary-color);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.version-warning {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--warning-light);
  color: var(--warning-color);
  border-radius: var(--radius-md);
  font-size: 12px;
  margin-bottom: 12px;
}

/* ===== 审查卡片 ===== */
.import-review-card {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 16px;
  margin-bottom: 12px;
}

.review-section {
  margin-bottom: 12px;
}
.review-section:last-child {
  margin-bottom: 0;
}

.review-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.review-label.collapsible {
  cursor: pointer;
  user-select: none;
}
.review-label.collapsible:hover {
  opacity: 0.7;
}

.collapse-icon {
  font-size: 10px;
  margin-left: 4px;
}

.review-content {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.7;
}

.review-content :deep(p) {
  margin: 0.5em 0;
}
.review-content :deep(h1),
.review-content :deep(h2),
.review-content :deep(h3),
.review-content :deep(h4),
.review-content :deep(h5),
.review-content :deep(h6) {
  margin: 0.8em 0 0.4em;
  font-weight: 600;
  font-size: 1em;
}
.review-content :deep(pre) {
  background: var(--code-bg, #0f172a);
  padding: 10px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}
.review-content :deep(pre code) {
  background: transparent;
  padding: 0;
  color: var(--code-text, #e2e8f0);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
}
.review-content :deep(code) {
  background: rgba(25, 118, 210, 0.12);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.9em;
}
.review-content :deep(ul),
.review-content :deep(ol) {
  padding-left: 20px;
  margin: 0.5em 0;
}
.review-content :deep(blockquote) {
  margin: 0.5em 0;
  padding-left: 10px;
  border-left: 3px solid var(--border-color);
  color: var(--text-secondary);
}
.review-content :deep(a) {
  color: var(--primary-color);
  text-decoration: underline;
}
.review-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}
.review-content :deep(th),
.review-content :deep(td) {
  border: 1px solid var(--border-color);
  padding: 6px 8px;
}
.review-content :deep(img) {
  max-width: 100%;
}
.review-content :deep(.katex) {
  font-size: 1.1em;
}
.review-content :deep(.katex-display) {
  display: block;
  text-align: center;
  margin: 0.5em 0;
  overflow-x: auto;
  overflow-y: hidden;
}
.review-content :deep(.katex-display > .katex) {
  font-size: 1.15em;
}

/* ===== 配置 ===== */
.review-config {
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  padding: 12px;
  margin-top: 12px;
}

.config-row {
  display: flex;
  gap: 12px;
}
.config-field {
  flex: 1;
}
.config-field label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 4px;
}
.required {
  color: var(--danger-color);
}

.type-select {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}

.apply-all-check {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
}

.duplicate-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  padding: 6px 10px;
  background: var(--warning-light);
  color: var(--warning-color);
  border-radius: var(--radius-sm);
  font-size: 12px;
}

/* ===== 操作按钮 ===== */
.review-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.counter {
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
}

.action-group {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;
}

.act-btn {
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border-color);
  transition: all 0.2s;
}
.act-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.skip-btn {
  background: var(--bg-secondary);
  color: var(--text-secondary);
}
.skip-btn:hover {
  background: var(--bg-tertiary);
}

.import-btn {
  background: var(--primary-color);
  color: #fff;
  border-color: var(--primary-color);
}
.import-btn:hover {
  opacity: 0.85;
}

.import-all-btn {
  background: var(--success-color);
  color: #fff;
  border-color: var(--success-color);
}
.import-all-btn:hover {
  opacity: 0.85;
}

/* ===== 结果 ===== */
.result-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 30px 20px;
}
.result-icon {
  font-size: 48px;
}
.result-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}
.result-detail {
  font-size: 14px;
  color: var(--text-secondary);
  text-align: center;
}

.result-errors {
  width: 100%;
  max-height: 150px;
  overflow-y: auto;
}
.result-error-item {
  padding: 4px 8px;
  font-size: 12px;
  color: var(--danger-color);
  background: var(--danger-light);
  border-radius: 4px;
  margin-bottom: 4px;
}

.result-icon.success {
  color: var(--success-color);
}
.result-icon.warning {
  color: var(--warning-color);
}
.result-icon.info {
  color: var(--info-color);
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
