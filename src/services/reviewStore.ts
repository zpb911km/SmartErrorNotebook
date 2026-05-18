// 复习队列共享状态
// Preview 设置队列 -> Review-Detail 消费

export interface ReviewCard {
  questionId: string
  srs: Record<string, any>
  question: Record<string, any>
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
