export interface Question {
  id: string
  questionType: QuestionType | null
  stem: string
  correctAnswer: string
  explanation: string | null
  note: string | null
  createdAt: string
  updatedAt: string
  sourceId: string | null
  tagIds: string[]
  attachmentIds: string[]
}

export type QuestionType =
  | 'SINGLE_SELECT'
  | 'MULTIPLE_SELECT'
  | 'TRUE_FALSE'
  | 'FILL_IN_THE_BLANK'
  | 'SHORT_ANSWER'
  | 'CALCULATION'
  | 'ESSAY'

export interface CreateQuestionRequest {
  sourceId?: string | null
  questionType?: QuestionType | null
  stem: string
  correctAnswer: string
  explanation?: string | null
  note?: string | null
  tagIds?: string[]
  attachmentIds?: string[]
}

export interface CreateQuestionResponse {
  question: Question
}

export interface UpdateQuestionRequest {
  id: string
  sourceId: string | null
  questionType: QuestionType | null
  stem: string
  correctAnswer: string
  explanation: string | null
  note: string | null
  tagIds: string[]
  attachmentIds: string[]
}

export interface UpdateQuestionResponse {
  question: Question
}

export interface DeleteQuestionRequest {
  id: string
}

export interface DeleteQuestionResponse {
  id: string
}

export interface GetQuestionRequest {
  id: string
}

export interface GetQuestionResponse {
  question: Question
}

export interface ListQuestionsRequest {
  filter?: QuestionFilter
  sort?: QuestionSort[]
  offset?: number
  limit?: number
}

export type QuestionSort =
  | 'UPDATED_AT_ASC'
  | 'UPDATED_AT_DESC'
  | 'MASTERY_ASC'
  | 'MASTERY_DESC'
  | 'ID_ASC'
  | 'ID_DESC'

export interface QuestionFilter {
  search?: string
  book?: string
  chapter?: string
  knowledge?: string
  tagIds?: string[]
  updatedSince?: string
  reviewState?: 'DUE' | 'NOT_DUE'
}

export interface ListQuestionsResponse {
  items: Question[]
  total: number
}
