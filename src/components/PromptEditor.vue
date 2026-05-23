<template>
  <div class="prompt-editor">
    <div
      class="editor-section"
      v-for="(section, index) in promptSections"
      :key="section.tag"
      :class="{ first: index === 0 }"
    >
      <div class="section-header">
        <div class="header-info">
          <span class="icon">{{ section.icon }}</span>
          <span class="title">{{ section.title }}</span>
        </div>
        <div class="header-actions">
          <button
            class="btn-restore"
            @click="restoreDefault(section.tag)"
            title="恢复默认提示词"
            :disabled="isRestoring"
          >
            恢复默认
          </button>
          <button
            class="btn-save"
            @click="savePrompt(section.tag)"
            :disabled="isSaving || !hasChanges(section.tag)"
          >
            保存
          </button>
        </div>
      </div>

      <div class="editor-container">
        <MarkdownTextarea
          v-model="tempPrompts[section.tag]"
          :show-preview="true"
          :default-view-mode="defaultViewModes[section.tag] || 'edit'"
          :textarea-class="'custom-prompt-input'"
          :preview-title="`${section.title} - 预览`"
          placeholder="输入提示词内容..."
        />
      </div>

      <div class="description">
        {{ section.description }}
      </div>
    </div>

    <!-- 保存状态提示 -->
    <div v-if="saveSuccess" class="save-success">✅ 提示词已保存</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MarkdownTextarea from './MarkdownTextarea.vue'

// ==================== 提示词配置 ====================

interface PromptSection {
  tag: string
  title: string
  icon: string
  description: string
  defaultPrompt: string
}

const promptSections: PromptSection[] = [
  {
    tag: 'question_text',
    title: '题干提取',
    icon: '📄',
    description:
      '用于从图片中提取题干内容，包括问题、选项、补充信息、示意图等。不要包含答案和手写信息。',
    defaultPrompt:
      '**请提取图片中的题干内容，以 Markdown 格式返回。**\n\n这是一些题目和答案的图片，注意区分题干，手写作答，和答案的图片。\n其中，请观察带有题干信息的图片 (一般是前一张或若干张图片，题干包含问题，选项，补充信息，示意图等)。\n题干只是学生做题时可以看见的部分，不包括图片中的答案和手写信息。\n注意公式使用 `$` 或者 `$$` 包裹\n请提取图片中的题干内容。'
  },
  {
    tag: 'answer',
    title: '答案生成',
    icon: '✅',
    description:
      '用于提取或生成题目的正确答案和解析。如果图片中有答案则提取原样内容，否则 AI 生成并标注"AI 生成答案"。',
    defaultPrompt:
      '这是一些题目和答案的图片，请观察全部图片，并区分题干，手写作答，答案的图片。\n\n如果存在答案的图片，请提取图片中的答案和解析内容，并以 Markdown 格式返回;\n否则，尝试作答并给出该题的正确答案和解析，标注 ** "AI 生成答案"**，以 Markdown 格式返回。\n\n注意 **只** 输出图片中*答案*和*解析*部分的原样内容，不用给出*题干*;\n注意公式使用 `$` 或者 `$$` 包裹'
  },
  {
    tag: 'analysis',
    title: '错误分析',
    icon: '💡',
    description:
      '用于分析错题的错误原因。切中要害进行分析，或给更高深而精悍的点拨，无需照抄图片内容。',
    defaultPrompt:
      '这是一些题目和答案的图片，请观察全部图片.\n\n请分析错题的错误原因。\n你无需照抄图片内容，只需要切中要害进行分析即可，或者给更高深而精悍的点拨\n以 Markdown 格式返回。'
  }
]

// ==================== 存储键 ====================
const STORAGE_KEY = 'custom_ai_prompts'

// ==================== 响应式数据 ====================

const customPrompts = ref<Record<string, string>>({})
const tempPrompts = ref<Record<string, string>>({})
const defaultViewModes = ref<Record<string, 'edit' | 'preview'>>({
  question_text: 'edit',
  answer: 'edit',
  analysis: 'edit'
})

const isSaving = ref(false)
const isRestoring = ref(false)
const saveSuccess = ref(false)

// ==================== 计算属性 ====================

const hasChanges = (tag: string) => {
  const original = customPrompts.value[tag] || getDefaultPrompt(tag)
  return tempPrompts.value[tag] !== original
}

// ==================== 生命周期 ====================

onMounted(() => {
  loadCustomPrompts()
})

// ==================== 方法 ====================

/**
 * 从 localStorage 加载自定义提示词
 */
const loadCustomPrompts = () => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      customPrompts.value = JSON.parse(stored)
      // 初始化临时变量
      promptSections.forEach((section) => {
        if (!tempPrompts.value[section.tag]) {
          tempPrompts.value[section.tag] =
            customPrompts.value[section.tag] || section.defaultPrompt
        }
      })
    } else {
      // 首次使用，用默认值填充
      promptSections.forEach((section) => {
        tempPrompts.value[section.tag] = section.defaultPrompt
      })
    }
  } catch (error) {
    console.error('加载自定义提示词失败:', error)
    // 出错时使用默认值
    promptSections.forEach((section) => {
      tempPrompts.value[section.tag] = section.defaultPrompt
    })
  }
}

/**
 * 获取默认提示词
 */
const getDefaultPrompt = (tag: string): string => {
  const section = promptSections.find((s) => s.tag === tag)
  return section?.defaultPrompt || ''
}

/**
 * 保存单个提示词
 */
const savePrompt = async (tag: string) => {
  isSaving.value = true
  try {
    customPrompts.value[tag] = tempPrompts.value[tag]
    localStorage.setItem(STORAGE_KEY, JSON.stringify(customPrompts.value))

    // 显示成功提示
    saveSuccess.value = true
    setTimeout(() => {
      saveSuccess.value = false
    }, 2000)
  } catch (error) {
    console.error('保存提示词失败:', error)
    alert('保存失败，请重试')
  } finally {
    isSaving.value = false
  }
}

/**
 * 恢复默认提示词
 */
const restoreDefault = async (tag: string) => {
  if (!confirm(`确定要恢复"${tag}"的默认提示词吗？`)) {
    return
  }

  isRestoring.value = true
  try {
    const defaultPrompt = getDefaultPrompt(tag)
    tempPrompts.value[tag] = defaultPrompt

    // 从自定义提示词中删除
    delete customPrompts.value[tag]
    localStorage.setItem(STORAGE_KEY, JSON.stringify(customPrompts.value))
  } catch (error) {
    console.error('恢复默认提示词失败:', error)
    alert('恢复失败，请重试')
  } finally {
    isRestoring.value = false
  }
}
</script>

<style scoped>
.prompt-editor {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  margin-top: 20px;
}

.editor-section {
  border-bottom: 1px solid var(--border-color);
  padding: 20px;
  transition: background 0.2s;
}

.editor-section:first-child {
  border-top-left-radius: 8px;
  border-top-right-radius: 8px;
}

.editor-section:last-child {
  border-bottom: none;
  border-bottom-left-radius: 8px;
  border-bottom-right-radius: 8px;
}

.editor-section:not(:first-child) {
  margin-top: 20px;
  padding-top: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon {
  font-size: 24px;
}

.title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.btn-restore {
  padding: 8px 16px;
  background: var(--input-bg);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-restore:hover:not(:disabled) {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.btn-restore:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-save {
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-save:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.editor-container {
  margin-bottom: 12px;
}

.custom-prompt-input :deep(.markdown-textarea__input) {
  min-height: 200px;
  font-family: 'SF Mono', Monaco, 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.description {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 12px;
  padding: 12px;
  background: var(--input-bg);
  border-radius: 8px;
  border-left: 3px solid var(--border-color);
}

.save-success {
  position: fixed;
  bottom: 24px;
  right: 24px;
  background: var(--primary-color);
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  font-size: 14px;
  animation: slideIn 0.3s ease;
  z-index: 1000;
}

@keyframes slideIn {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}
</style>
