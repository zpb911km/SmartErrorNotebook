export interface SrsData {
  questionId: string
  stability: number
  difficulty: number
  retrievability: number
  nextReviewAt: string | null
  lastReviewAt: string | null
  reviewCount: number
  isDue: boolean
}

export interface LibraryStatisticsData {
  questionTotal: number
  cardTotal: number
  dueCount: number
  newCardCount: number
  averageStability: number
  averageDifficulty: number
  totalReviews: number
}

export interface SubmitReviewRequest {
  questionId: string
  feedback: number
  reviewedAt: string
}

export interface SubmitReviewResponse {
  srs: SrsData
  nextIntervalDays: number
}

export interface ResetReviewProgressRequest {
  questionId: string
  resetAt: string
}

export interface ResetReviewProgressResponse {
  srs: SrsData
}

export interface GetSrsDataRequest {
  questionId: string
  at: string
}

export interface GetSrsDataResponse {
  srs: SrsData
}

export interface ListSrsDataRequest {
  at: string
}

export interface ListSrsDataResponse {
  items: SrsData[]
}

export interface GetLibraryStatisticsRequest {
  at: string
}

export interface GetLibraryStatisticsResponse {
  statistics: LibraryStatisticsData
}
