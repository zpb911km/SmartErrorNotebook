import { beforeEach, expect, it, vi } from 'vitest'
import {
  parseImportFile,
  importSingleQuestion,
  getExistingPromptSet
} from '../src/utils/importJson'
import { exportQuestionsToJSON } from '../src/utils/exportJson'
import { exportFile } from '../src/utils/exportFile'
import { loadQuestionLibrary } from '../src/services/questionQueries'
import { createQuestionEditor } from '../src/services/questionEditor'
import { buildQuestionsHTML } from '../src/utils/exportHtml'
import { parseQuestionType } from '../src/utils/questionDisplay'

vi.mock('../src/utils/exportFile', () => ({
  exportFile: vi.fn(async () => true)
}))
vi.mock('../src/utils/notification', () => ({ showError: vi.fn() }))
vi.mock('../src/services/questionQueries', () => ({
  loadQuestionLibrary: vi.fn()
}))
vi.mock('../src/services/questionEditor', () => ({
  createQuestionEditor: vi.fn()
}))
beforeEach(() => {
  vi.clearAllMocks()
})

it('exports canonical content to the unchanged v1 wire format and imports it again', async () => {
  await exportQuestionsToJSON([
    { stem: '题干', correctAnswer: '答案', explanation: null }
  ])
  const wire = String(vi.mocked(exportFile).mock.calls[0][1])
  const data = JSON.parse(wire)
  expect(data).toMatchObject({
    version: '1.0',
    count: 1,
    questions: [{ prompt: '题干', answer: '答案', analysis: '' }]
  })
  expect(data.questions[0]).not.toHaveProperty('stem')
  expect(parseImportFile(wire)).toEqual({
    questions: data.questions,
    version: '1.0'
  })
})

it('rejects malformed input and defaults missing optional text', () => {
  expect(parseImportFile('{').error).toBeTruthy()
  expect(
    parseImportFile('{"version":"1.0","questions":[{"prompt":42}]}').error
  ).toBeTruthy()
  expect(
    parseImportFile('{"version":"1.0","questions":[{"prompt":"题干"}]}')
      .questions
  ).toEqual([{ prompt: '题干', answer: '', analysis: '' }])
})

it('imports through the editor with Current fields and surfaces failures', async () => {
  const save = vi
    .fn()
    .mockRejectedValueOnce(new Error('storage'))
    .mockResolvedValueOnce({ id: 'question' })
  vi.mocked(createQuestionEditor).mockReturnValue({
    save,
    reset: vi.fn(),
    retryCleanup: vi.fn()
  })
  const row = { prompt: '题干', answer: '答案', analysis: '解析' }
  expect(
    (
      await importSingleQuestion(row, 'subject', '简答题', [
        { name: '计算', color: '#fff' }
      ])
    ).success
  ).toBe(false)
  expect(
    (await importSingleQuestion(row, 'subject', 'SHORT_ANSWER')).success
  ).toBe(true)
  expect(createQuestionEditor).toHaveBeenCalledTimes(1)
  expect(save).toHaveBeenLastCalledWith(
    expect.objectContaining({
      stem: '题干',
      correctAnswer: '答案',
      explanation: '解析',
      questionType: 'SHORT_ANSWER',
      source: {
        kind: 'new',
        subjectId: 'subject',
        book: null,
        chapter: null,
        knowledge: null
      }
    })
  )
})

it('propagates lookup failures rather than pretending no duplicates exist', async () => {
  vi.mocked(loadQuestionLibrary).mockRejectedValueOnce(new Error('unavailable'))
  await expect(getExistingPromptSet()).rejects.toThrow('unavailable')
})

it('renders HTML exports from Current content and respects answer visibility', () => {
  const content = [
    {
      stem: 'UNIQUE_STEM',
      correctAnswer: 'UNIQUE_ANSWER',
      explanation: 'UNIQUE_EXPLANATION'
    }
  ]
  expect(buildQuestionsHTML(content, false)).toContain('UNIQUE_STEM')
  expect(buildQuestionsHTML(content, false)).not.toContain('UNIQUE_ANSWER')
  expect(buildQuestionsHTML(content, true)).toContain('UNIQUE_EXPLANATION')
})

it('maps UI and AI labels but rejects unknown enum values', () => {
  expect(parseQuestionType('多选题')).toBe('MULTIPLE_SELECT')
  expect(parseQuestionType('TRUE_FALSE')).toBe('TRUE_FALSE')
  expect(parseQuestionType('')).toBeNull()
  expect(() => parseQuestionType('unknown')).toThrow('未知题型')
})
