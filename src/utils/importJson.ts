import { readTextFile } from '@tauri-apps/plugin-fs'
import {
  createErrorQuestion
} from '../apis/errorQuestions'
import { getQuestions } from '../apis/errorQuestions'
import type {
  ExportJSONSchema,
  ErrorQuestion,
  ImportResult
} from '../types'
import { showSuccess, showWarning } from './notification'

/**
 * 从 JSON 文件读取并校验导入数据
 * @returns 解析后的 ExportJSONSchema，校验失败则抛出错误
 */
export async function readImportFile(filePath: string): Promise<ExportJSONSchema> {
  // 1. 检查文件扩展名
  if (!filePath.toLowerCase().endsWith('.json')) {
    throw new Error('请选择 .json 格式的文件')
  }

  // 2. 读取文件内容
  let content: string
  try {
    content = await readTextFile(filePath)
  } catch (error) {
    throw new Error(`无法读取文件: ${String(error)}`)
  }

  // 3. 解析 JSON
  let data: any
  try {
    data = JSON.parse(content)
  } catch (error) {
    throw new Error('文件不是有效的 JSON 格式')
  }

  // 4. 校验结构
  if (!data || typeof data !== 'object') {
    throw new Error('JSON 数据必须是对象')
  }

  // 4.1 校验 version
  if (!data.version) {
    throw new Error('缺少必要的 "version" 字段')
  }

  // 版本不匹配时警告但允许继续
  if (data.version !== '1.0') {
    showWarning(
      '版本不匹配',
      `导入文件版本为 "${data.version}"，当前支持版本为 "1.0"，可能会存在兼容性问题`
    )
  }

  // 4.2 校验 questions
  if (!Array.isArray(data.questions)) {
    throw new Error('"questions" 必须是数组')
  }

  if (data.questions.length === 0) {
    throw new Error('"questions" 数组不能为空')
  }

  // 4.3 校验每条记录
  const errors: string[] = []
  data.questions.forEach((q: any, index: number) => {
    if (!q || typeof q !== 'object') {
      errors.push(`第 ${index + 1} 条记录不是有效对象`)
      return
    }
    if (!q.prompt || typeof q.prompt !== 'string') {
      errors.push(`第 ${index + 1} 条记录缺少 "prompt" 字段`)
    }
    // answer 和 analysis 可以为空字符串
    if (q.answer !== undefined && typeof q.answer !== 'string') {
      errors.push(`第 ${index + 1} 条记录的 "answer" 必须是字符串`)
    }
    if (q.analysis !== undefined && typeof q.analysis !== 'string') {
      errors.push(`第 ${index + 1} 条记录的 "analysis" 必须是字符串`)
    }
  })

  if (errors.length > 0) {
    throw new Error(`数据校验失败:\n${errors.join('\n')}`)
  }

  return data as ExportJSONSchema
}

/**
 * 执行导入操作
 * @param data 解析后的导入数据
 * @param subjectId 用户选择的目标科目 ID
 * @param defaultType 默认题型
 * @param userId 当前用户 ID
 * @returns 导入结果统计
 */
export async function importQuestions(
  data: ExportJSONSchema,
  subjectId: string,
  defaultType: string,
  userId: string
): Promise<ImportResult> {
  const result: ImportResult = {
    success: 0,
    skipped: 0,
    failed: 0,
    errors: []
  }

  // 获取本地已有题目的 prompt 集合，用于去重
  const existingPrompts = new Set<string>()
  try {
    const existingQuestions: ErrorQuestion[] = await getQuestions()
    existingQuestions.forEach((q) => {
      if (q.prompt) {
        existingPrompts.add(q.prompt.trim())
      }
    })
  } catch (error) {
    console.warn('获取本地题目列表失败，将跳过去重检查:', error)
  }

  for (let i = 0; i < data.questions.length; i++) {
    const q = data.questions[i]

    try {
      // 去重：以 prompt 文本精确匹配
      if (existingPrompts.has(q.prompt.trim())) {
        result.skipped++
        continue
      }

      // 创建错题（API 会自动生成新 UUID）
      await createErrorQuestion({
        user_id: userId,
        subject_id: subjectId,
        source_id: undefined,
        prompt: q.prompt,
        type: defaultType as any,
        answer: q.answer || '',
        analysis: q.analysis || '',
        error_note: ''
      })

      // 加入已导入集合，防止后续重复
      existingPrompts.add(q.prompt.trim())
      result.success++
    } catch (error) {
      result.failed++
      result.errors.push(`第 ${i + 1} 题导入失败: ${String(error)}`)
    }
  }

  // 显示结果通知
  const parts: string[] = []
  if (result.success > 0) parts.push(`成功 ${result.success} 题`)
  if (result.skipped > 0) parts.push(`跳过 ${result.skipped} 题（重复）`)
  if (result.failed > 0) parts.push(`失败 ${result.failed} 题`)

  if (result.failed === 0) {
    showSuccess('导入完成', parts.join('，'))
  } else {
    showWarning('导入完成', `${parts.join('，')}\n${result.errors.join('\n')}`)
  }

  return result
}

/**
 * 获取当前用户 ID（从本地存储或配置中读取）
 * 由于没有专门的 user API，从 created_by 或固定值获取
 * 这里从 localStorage 读取，若无则使用默认值
 */
export function getCurrentUserId(): string {
  try {
    // 尝试从 localStorage 读取用户 ID
    const stored = localStorage.getItem('user_id')
    if (stored) return stored
  } catch {
    // ignore
  }

  // 从 URL 参数或固定默认值
  return 'default-user'
}
