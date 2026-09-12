import { reactive, ref } from 'vue'
import {
  createQuestionEditor,
  type SaveState,
  type QuestionDraft
} from '../services/questionEditor'
import { errorMessage } from '../utils/errors'

export function useQuestionEditor() {
  const state = reactive<SaveState>({
    stage: 'idle',
    committed: null,
    cleanupIds: [],
    error: null
  })
  const busy = ref(false)
  const editor = createQuestionEditor(state)
  const save = async (draft: QuestionDraft, originalIds: string[] = []) => {
    if (busy.value) throw new Error('保存正在进行')
    busy.value = true
    try {
      return await editor.save(draft, originalIds)
    } finally {
      busy.value = false
    }
  }
  const retryCleanup = async () => {
    if (busy.value) return
    busy.value = true
    try {
      await editor.retryCleanup()
    } finally {
      busy.value = false
    }
  }
  const failureMessage = () => {
    const labels = {
      idle: '准备数据',
      source: '保存来源',
      tags: '保存标签',
      attachments: '保存图片',
      question: '保存题目',
      cleanup: '清理旧图片',
      saved: '刷新详情'
    }
    return `${labels[state.stage]}失败：${errorMessage(state.error)}。已完成部分将在重试时复用。`
  }
  return {
    state,
    busy,
    save,
    retryCleanup,
    reset: editor.reset,
    failureMessage
  }
}
