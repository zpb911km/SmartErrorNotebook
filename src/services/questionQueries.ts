import {
  getQuestion,
  listQuestions,
  listSources,
  listSubjects,
  listTags,
  listSrsData
} from '../api'
import type {
  ListQuestionsRequest,
  Question,
  Source,
  Subject,
  Tag,
  SrsData
} from '../types'
import type { QuestionView } from '../types/questionView'

export interface QuestionResources {
  sources: Source[]
  subjects: Subject[]
  tags: Tag[]
  srs: SrsData[]
}

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

export async function loadQuestionResources(
  at = new Date().toISOString()
): Promise<QuestionResources> {
  const [sources, subjects, tags, srs] = await Promise.all([
    listSources(),
    listSubjects(),
    listTags(),
    listSrsData(at)
  ])
  return { sources, subjects, tags, srs }
}

export async function loadQuestionLibrary(
  request: ListQuestionsRequest = {},
  subjectId?: string
) {
  // Subject is a Source projection and must be filtered before client pagination.
  const [page, resources] = await Promise.all([
    listQuestions({
      ...request,
      sort: request.sort ?? ['UPDATED_AT_DESC'],
      ...(subjectId ? { offset: undefined, limit: undefined } : {})
    }),
    loadQuestionResources()
  ])
  let items = projectQuestions(page.items, resources)
  let total = page.total
  if (subjectId) {
    items = items.filter((item) => item.subject?.id === subjectId)
    total = items.length
    const offset = request.offset ?? 0
    items = items.slice(
      offset,
      request.limit === undefined ? undefined : offset + request.limit
    )
  }
  return { ...resources, items, total }
}

export async function loadQuestionDetail(id: string) {
  const [question, resources] = await Promise.all([
    getQuestion(id),
    loadQuestionResources()
  ])
  return { ...resources, question: projectQuestions([question], resources)[0] }
}
