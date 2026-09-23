import {
  getQuestion,
  listQuestions,
  listSources,
  listSrsData,
  listSubjects,
  listTags
} from '../api'
import type {
  ListQuestionsRequest,
  Question,
  Source,
  SrsData,
  Subject,
  Tag
} from '../types'
import type { QuestionView } from '../types/questionView'

export interface QuestionResources {
  sources: Source[]
  subjects: Subject[]
  tags: Tag[]
  srs: SrsData[]
}

export type QuestionMetadata = Pick<
  QuestionResources,
  'sources' | 'subjects' | 'tags'
>

export function projectQuestions(
  questions: Question[],
  resources: QuestionResources
): QuestionView[] {
  const sources = new Map(resources.sources.map((item) => [item.id, item]))
  const subjects = new Map(resources.subjects.map((item) => [item.id, item]))
  const tags = new Map(resources.tags.map((item) => [item.id, item]))
  const srs = new Map(resources.srs.map((item) => [item.questionId, item]))
  return questions.map((question) => {
    const source = sources.get(question.sourceId ?? '') ?? null
    return {
      ...question,
      source,
      subject: subjects.get(source?.subjectId ?? '') ?? null,
      tags: question.tagIds.flatMap((id) =>
        tags.has(id) ? [tags.get(id)!] : []
      ),
      srs: srs.get(question.id) ?? null
    }
  })
}

export async function loadQuestionMetadata(): Promise<QuestionMetadata> {
  const [sources, subjects, tags] = await Promise.all([
    listSources(),
    listSubjects(),
    listTags()
  ])
  return { sources, subjects, tags }
}

export async function loadQuestionResources(
  at = new Date().toISOString()
): Promise<QuestionResources> {
  const [metadata, srs] = await Promise.all([
    loadQuestionMetadata(),
    listSrsData(at)
  ])
  return { ...metadata, srs }
}

export async function loadQuestionLibrary(request: ListQuestionsRequest = {}) {
  const [page, resources] = await Promise.all([
    listQuestions({
      ...request,
      sort: request.sort ?? ['UPDATED_AT_DESC']
    }),
    loadQuestionResources()
  ])
  return {
    ...resources,
    items: projectQuestions(page.items, resources),
    total: page.total
  }
}

export async function loadQuestionDetail(id: string) {
  const [question, resources] = await Promise.all([
    getQuestion(id),
    loadQuestionResources()
  ])
  return { ...resources, question: projectQuestions([question], resources)[0] }
}
