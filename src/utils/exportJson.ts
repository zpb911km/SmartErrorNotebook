import type { QuestionContent } from '../types/questionView'
import type { ExportJSONSchema } from '../types/transfer'
import { exportFile } from './exportFile'
import { showError } from './notification'

export async function exportQuestionsToJSON(
  questions: QuestionContent[]
): Promise<boolean> {
  try {
    if (questions.length === 0) {
      showError('导出失败', '当前筛选条件下没有可导出的错题')
      return false
    }

    const exportData: ExportJSONSchema = {
      version: '1.0',
      exportedAt: new Date().toISOString(),
      count: questions.length,
      questions: questions.map((q) => ({
        prompt: q.stem || '',
        answer: q.correctAnswer || '',
        analysis: q.explanation || ''
      }))
    }

    const jsonString = JSON.stringify(exportData, null, 2)
    const filename = `错题导出_${new Date().toISOString().slice(0, 10)}.json`

    return await exportFile(filename, jsonString, 'application/json')
  } catch (error) {
    console.error('导出 JSON 失败:', error)
    showError('导出失败', `导出过程中出现错误: ${String(error)}`)
    return false
  }
}
