import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import type { ErrorQuestion, ExportJSONSchema } from '../types'
import { showError, showSuccess } from './notification'

/**
 * 将错题数据导出为 JSON 文件
 * @param questions 已筛选的错题列表（来自前端已过滤数据）
 * @returns 是否成功导出
 */
export async function exportQuestionsToJSON(
  questions: ErrorQuestion[]
): Promise<boolean> {
  try {
    if (questions.length === 0) {
      showError('导出失败', '当前筛选条件下没有可导出的错题')
      return false
    }

    // 构建导出数据（只取 prompt、answer、analysis 三个字段）
    const exportData: ExportJSONSchema = {
      version: '1.0',
      exportedAt: new Date().toISOString(),
      count: questions.length,
      questions: questions.map((q) => ({
        prompt: q.prompt || '',
        answer: q.answer || '',
        analysis: q.analysis || ''
      }))
    }

    // 序列化为 JSON 字符串（缩进 2 格）
    const jsonString = JSON.stringify(exportData, null, 2)

    // 弹出保存对话框让用户选择路径
    const filePath = await save({
      defaultPath: `错题导出_${new Date().toISOString().slice(0, 10)}.json`,
      filters: [
        {
          name: 'JSON 文件',
          extensions: ['json']
        }
      ]
    })

    if (!filePath) {
      // 用户取消了保存
      return false
    }

    // 写入文件（UTF-8 编码）
    await writeTextFile(filePath, jsonString)

    showSuccess('导出成功', `已导出 ${questions.length} 道错题到 ${filePath}`)
    return true
  } catch (error) {
    console.error('导出 JSON 失败:', error)
    showError('导出失败', `导出过程中出现错误: ${String(error)}`)
    return false
  }
}
