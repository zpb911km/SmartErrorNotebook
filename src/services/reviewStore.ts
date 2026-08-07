// 复习队列共享状态
// Preview 设置队列 -> Review-Detail 消费

import type { SRSData, ErrorQuestion } from '../types'

export interface ReviewCard {
  questionId: string
  srs: SRSData
  question: Partial<ErrorQuestion>
  subjectName: string
}

let _queue: ReviewCard[] = []
let _onQueue: 'due' | 'all' = 'due'

export function setReviewQueue(
  queue: ReviewCard[],
  scope: 'due' | 'all' = 'due'
) {
  _queue = queue
  _onQueue = scope
}

export function getReviewQueue(): ReviewCard[] {
  return _queue
}

export function getReviewScope(): 'due' | 'all' {
  return _onQueue
}

export function clearReviewQueue() {
  _queue = []
}
