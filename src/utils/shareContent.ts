import { appCacheDir, join } from '@tauri-apps/api/path'
import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'
import { shareFile } from 'tauri-plugin-share'
import { showError } from './notification'
import type { ErrorQuestion } from '../types'
import { buildQuestionsHTML } from './exportHtml'

/**
 * 通用分享流程：写入临时文件 → 调用系统分享面板
 *
 * 仅在移动端（iOS/Android）可用，桌面端不显示分享按钮。
 *
 * @param filename  文件名（含扩展名）
 * @param content   文本内容
 * @param mimeType  MIME 类型
 * @returns 是否成功
 */
export async function shareContent(
  filename: string,
  content: string,
  mimeType: string
): Promise<boolean> {
  try {
    // 1. 写入到 AppCache 目录（使用 baseDir 匹配 scope 权限）
    await writeTextFile(filename, content, { baseDir: BaseDirectory.AppCache })

    // 2. 拼接绝对路径给 shareFile
    const cacheDir = await appCacheDir()
    const filePath = await join(cacheDir, filename)

    // 3. 调用系统分享面板
    const result = await shareFile(filePath, mimeType)
    if (result !== null) {
      console.warn('分享返回非空结果:', result)
    }
    return true
  } catch (e: any) {
    console.error('分享失败:', e)
    showError('分享失败', `分享过程中出现错误: ${String(e)}`)
    return false
  }
}

/**
 * 分享为 JSON 格式
 */
export async function shareQuestionsToJSON(
  questions: ErrorQuestion[]
): Promise<boolean> {
  if (questions.length === 0) {
    showError('分享失败', '当前筛选条件下没有可分享的错题')
    return false
  }

  const exportData = {
    version: '1.0',
    exportedAt: new Date().toISOString(),
    count: questions.length,
    questions: questions.map((q) => ({
      prompt: q.prompt || '',
      answer: q.answer || '',
      analysis: q.analysis || ''
    }))
  }

  const jsonString = JSON.stringify(exportData, null, 2)
  const filename = `sen_share_${Date.now()}.json`

  return await shareContent(filename, jsonString, 'application/json')
}

/**
 * 分享为 HTML 格式（复用 exportHtml 中的构建逻辑）
 */
export async function shareQuestionsToHTML(
  questions: ErrorQuestion[]
): Promise<boolean> {
  if (questions.length === 0) {
    showError('分享失败', '当前筛选条件下没有可分享的错题')
    return false
  }

  try {
    const html = buildQuestionsHTML(questions)
    const filename = `sen_share_${Date.now()}.html`
    return await shareContent(filename, html, 'text/html;charset=utf-8')
  } catch (e: any) {
    console.error('分享 HTML 失败:', e)
    showError('分享失败', `HTML 分享过程中出现错误: ${String(e)}`)
    return false
  }
}
