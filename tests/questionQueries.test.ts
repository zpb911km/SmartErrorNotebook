import { beforeEach, describe, expect, it, vi } from 'vitest'

import * as api from '../src/api'
import {
  loadQuestionDetail,
  loadQuestionLibrary,
  projectQuestions
} from '../src/services/questionQueries'
import type { Question, Source, SrsData, Subject } from '../src/types'

vi.mock('../src/api', () => ({
  listQuestions: vi.fn(),
  getQuestion: vi.fn(),
  listSources: vi.fn(),
  listSubjects: vi.fn(),
  listTags: vi.fn(),
  listSrsData: vi.fn()
}))

const at = '2026-09-11T00:00:00Z'
const subject: Subject = {
  id: 'subject',
  name: '数学',
  color: '#fff',
  createdAt: at,
  updatedAt: at
}
const source: Source = {
  id: 'source',
  subjectId: subject.id,
  book: null,
  chapter: null,
  knowledge: null,
  createdAt: at,
  updatedAt: at
}
const question = (
  id: string,
  sourceId: string | null = source.id
): Question => ({
  id,
  sourceId,
  stem: id,
  correctAnswer: '',
  explanation: null,
  note: null,
  questionType: null,
  tagIds: ['tag', 'missing'],
  attachmentIds: ['attachment'],
  createdAt: at,
  updatedAt: at
})
const srs: SrsData = {
  questionId: 'first',
  stability: 1,
  difficulty: 5,
  retrievability: 0.5,
  nextReviewAt: at,
  lastReviewAt: null,
  reviewCount: 0,
  isDue: true
}

beforeEach(() => {
  vi.resetAllMocks()
  vi.mocked(api.listSources).mockResolvedValue([source])
  vi.mocked(api.listSubjects).mockResolvedValue([subject])
  vi.mocked(api.listTags).mockResolvedValue([
    { id: 'tag', name: '计算', color: '#000' }
  ])
  vi.mocked(api.listSrsData).mockResolvedValue([srs])
})

describe('Current question queries', () => {
  it('loads relationships once per batch and preserves canonical fields', async () => {
    vi.mocked(api.listQuestions).mockResolvedValue({
      items: [question('first'), question('second')],
      total: 2
    })
    const library = await loadQuestionLibrary()
    expect(library.items[0]).toMatchObject({
      stem: 'first',
      subject,
      source,
      srs,
      tags: [{ id: 'tag', name: '计算', color: '#000' }]
    })
    expect(library.items[0]).not.toHaveProperty('prompt')
    expect(library.items[0]).not.toHaveProperty('subjectId')
    expect(api.listSubjects).toHaveBeenCalledTimes(1)
    expect(api.listSrsData).toHaveBeenCalledTimes(1)
    expect(library.items[1].srs).toBeNull()
  })

  it('filters the full subject candidate set before pagination', async () => {
    vi.mocked(api.listQuestions).mockResolvedValue({
      items: [question('other', null), question('first'), question('second')],
      total: 3
    })
    const library = await loadQuestionLibrary(
      { offset: 1, limit: 1, filter: { search: 'test' } },
      subject.id
    )
    expect(api.listQuestions).toHaveBeenCalledWith(
      expect.objectContaining({
        offset: undefined,
        limit: undefined,
        filter: { search: 'test' }
      })
    )
    expect(library.total).toBe(2)
    expect(library.items.map((item) => item.id)).toEqual(['second'])
  })

  it('preserves backend pagination when no client subject filter is requested', async () => {
    vi.mocked(api.listQuestions).mockResolvedValue({
      items: [question('first')],
      total: 9
    })
    const result = await loadQuestionLibrary({
      offset: 3,
      limit: 1,
      sort: ['ID_ASC']
    })
    expect(api.listQuestions).toHaveBeenCalledWith({
      offset: 3,
      limit: 1,
      sort: ['ID_ASC']
    })
    expect(result.total).toBe(9)
  })

  it('handles empty lists and missing resources without inventing entities or writing SRS', () => {
    const resources = { sources: [], subjects: [], tags: [], srs: [] }
    expect(projectQuestions([], resources)).toEqual([])
    expect(projectQuestions([question('missing')], resources)[0]).toMatchObject(
      { source: null, subject: null, tags: [], srs: null }
    )
  })

  it('loads detail without an attachment fetch or a second question read', async () => {
    vi.mocked(api.getQuestion).mockResolvedValue(question('first'))
    const result = await loadQuestionDetail('first')
    expect(result.question.attachmentIds).toEqual(['attachment'])
    expect(api.getQuestion).toHaveBeenCalledTimes(1)
    expect(api.listQuestions).not.toHaveBeenCalled()
  })
})
