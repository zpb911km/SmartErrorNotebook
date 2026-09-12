import type { QuestionType } from '../types'

export const questionTypeLabels: Record<QuestionType, string> = {
  SINGLE_SELECT: '单选题',
  MULTIPLE_SELECT: '多选题',
  TRUE_FALSE: '判断题',
  FILL_IN_THE_BLANK: '填空题',
  SHORT_ANSWER: '简答题',
  CALCULATION: '计算题',
  ESSAY: '论述题'
}

export const questionTypeLabel = (type: QuestionType | null) =>
  type ? questionTypeLabels[type] : ''

/** The AI and human-facing form may supply a translated label. */
export function parseQuestionType(value: string | null): QuestionType | null {
  if (!value) return null
  if (Object.prototype.hasOwnProperty.call(questionTypeLabels, value))
    return value as QuestionType
  const entry = Object.entries(questionTypeLabels).find(
    ([, label]) => label === value
  )
  if (!entry) throw new Error(`未知题型：${value}`)
  return entry[0] as QuestionType
}

export const timestampSeconds = (value: string | null | undefined) =>
  value ? new Date(value).getTime() / 1000 : 0

export const formatTimestamp = (
  value: string | null | undefined,
  emptyText = '-'
) => (value ? new Date(value).toLocaleString('zh-CN') : emptyText)
