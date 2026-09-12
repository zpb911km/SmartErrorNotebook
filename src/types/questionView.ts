import type { Question, Source, Subject, Tag, SrsData } from './index'

/** Read-only projection; persisted fields always come from the Current API. */
export interface QuestionView extends Question {
  source: Source | null
  subject: Subject | null
  tags: Tag[]
  srs: SrsData | null
}

export type QuestionContent = Pick<
  Question,
  'stem' | 'correctAnswer' | 'explanation'
> & { id?: string }
