// 社区分享暂存数据
// Community 设置 -> Add 消费

import type { Question } from '../types'
import type { QuestionContent } from '../types/questionView'

export type SharedQuestionData = QuestionContent &
  Pick<Question, 'questionType' | 'note'>

let _sharedData: SharedQuestionData | null = null

export function setSharedData(data: SharedQuestionData) {
  _sharedData = data
}

export function getSharedData(): SharedQuestionData | null {
  return _sharedData
}

export function clearSharedData() {
  _sharedData = null
}
