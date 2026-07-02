import { readTextFile } from '@tauri-apps/plugin-fs'
import { createErrorQuestion } from '../apis/errorQuestions'
import { createErrorTagsForQuestion } from '../apis/errorTags'
import { createSRSData } from '../apis/srsData'
import { getQuestions } from '../apis/errorQuestions'
import type { ExportJSONSchema, ErrorQuestion, ImportResult } from '../types'
import { showSuccess, showWarning } from './notification'

/** 解析并校验 JSON 文件，返回题目列表和错误信息 */
export function parseImportFile(content: string): {
  questions: ExportJSONSchema['questions']
  version: string
  error?: string
} {
  let data: any
  try {
    data = JSON.parse(content)
  } catch {
    return { questions: [], version: '', error: '文件不是有效的 JSON 格式' }
  }

  if (!data || typeof data !== 'object') {
    return { questions: [], version: '', error: 'JSON 数据必须是对象' }
  }
  if (!data.version) {
    return { questions: [], version: '', error: '缺少必要的 "version" 字段' }
  }
  if (!Array.isArray(data.questions)) {
    return { questions: [], version: '', error: '"questions" 必须是数组' }
  }
  if (data.questions.length === 0) {
    return { questions: [], version: '', error: '"questions" 数组不能为空' }
  }

  const errs: string[] = []
  data.questions.forEach((q: any, i: number) => {
    if (!q || typeof q !== 'object') {
      errs.push(`第 ${i + 1} 条记录不是有效对象`)
      return
    }
    if (!q.prompt || typeof q.prompt !== 'string') {
      errs.push(`第 ${i + 1} 条记录缺少 "prompt" 字段`)
    }
    if (q.answer !== undefined && typeof q.answer !== 'string') {
      errs.push(`第 ${i + 1} 条记录的 "answer" 必须是字符串`)
    }
    if (q.analysis !== undefined && typeof q.analysis !== 'string') {
      errs.push(`第 ${i + 1} 条记录的 "analysis" 必须是字符串`)
    }
  })

  if (errs.length > 0) {
    return { questions: [], version: '', error: `数据校验失败:\n${errs.join('\n')}` }
  }

  return { questions: data.questions, version: data.version }
}

/**
 * 导入单道题，包含 SRS 数据和错因标签的创建
 * @returns 是否成功
 */
export async function importSingleQuestion(
  question: ExportJSONSchema['questions'][0],
  subjectId: string,
  typeName: string,
  userId: string,
  tags?: Array<{ name: string; color: string }>
): Promise<{ success: boolean; error?: string }> {
  try {
    // 1. 创建错题
    const created = await createErrorQuestion({
      user_id: userId,
      subject_id: subjectId,
      source_id: undefined,
      prompt: question.prompt,
      type: typeName as any,
      answer: question.answer || '',
      analysis: question.analysis || '',
      error_note: ''
    })

    // 2. 自动创建 SRS 数据（默认中等难度 5.0）
    try {
      await createSRSData(created.id, 5.0)
    } catch (srsError) {
      console.warn('创建 SRS 数据失败（不影响导入）:', srsError)
    }

    // 3. 创建错因标签（如果有选）
    if (tags && tags.length > 0) {
      try {
        await createErrorTagsForQuestion(created.id, tags)
      } catch (tagError) {
        console.warn('创建错因标签失败（不影响导入）:', tagError)
      }
    }

    return { success: true }
  } catch (error) {
    return { success: false, error: String(error) }
  }
}

/**
 * 获取已有题目的 prompt 集合，用于去重判断
 */
export async function getExistingPromptSet(): Promise<Set<string>> {
  const set = new Set<string>()
  try {
    const existing: ErrorQuestion[] = await getQuestions()
    existing.forEach((q) => {
      if (q.prompt) set.add(q.prompt.trim())
    })
  } catch {
    console.warn('获取本地题目列表失败，跳过去重检查')
  }
  return set
}
