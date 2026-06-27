// 社区分享暂存数据
// Community 设置 -> Add 消费

export interface SharedQuestionData {
  prompt: string
  type_: string
  answer: string
  analysis: string
  error_note: string
}

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
