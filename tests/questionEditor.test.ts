import { beforeEach, describe, expect, it, vi } from 'vitest'

import * as api from '../src/api'
import {
  createQuestionEditor,
  type QuestionDraft,
  type SaveState
} from '../src/services/questionEditor'
import { materializeSourceSelection } from '../src/services/sourcePersistence'

vi.mock('../src/api', () => ({
  listTags: vi.fn(),
  createTag: vi.fn(),
  createAttachment: vi.fn(),
  createQuestion: vi.fn(),
  updateQuestion: vi.fn(),
  deleteAttachment: vi.fn()
}))
vi.mock('../src/services/sourcePersistence', () => ({
  materializeSourceSelection: vi.fn()
}))
vi.mock('../src/utils/attachments', () => ({
  prepareAttachment: vi.fn(async (base64Data: string) => ({
    base64Data,
    mimeType: 'image/png'
  }))
}))

function session() {
  const state: SaveState = {
    stage: 'idle',
    committed: null,
    cleanupIds: [],
    error: null
  }
  return { state, editor: createQuestionEditor(state) }
}
function draft(): QuestionDraft {
  return {
    source: { kind: 'none' },
    questionType: null,
    stem: '题目',
    correctAnswer: '',
    explanation: null,
    note: null,
    tags: [{ name: '计算', color: '#fff' }],
    attachments: [{ base64Data: 'first' }, { base64Data: 'second' }]
  }
}

beforeEach(() => {
  vi.resetAllMocks()
  vi.mocked(materializeSourceSelection).mockImplementation(
    async (selection) => ({ selection, sourceId: null })
  )
  vi.mocked(api.listTags).mockResolvedValue([])
  vi.mocked(api.createTag).mockResolvedValue({
    id: 'tag',
    name: '计算',
    color: '#fff'
  })
  vi.mocked(api.createAttachment).mockImplementation(
    async (request) => request.base64Data + '-id'
  )
  vi.mocked(api.createQuestion).mockImplementation(async (request) => ({
    id: 'question',
    createdAt: 'now',
    updatedAt: 'now',
    sourceId: request.sourceId ?? null,
    questionType: request.questionType ?? null,
    stem: request.stem,
    correctAnswer: request.correctAnswer,
    explanation: request.explanation ?? null,
    note: request.note ?? null,
    tagIds: request.tagIds ?? [],
    attachmentIds: request.attachmentIds ?? []
  }))
  vi.mocked(api.updateQuestion).mockImplementation(async (request) => ({
    ...request,
    createdAt: 'now',
    updatedAt: 'now'
  }))
  vi.mocked(api.deleteAttachment).mockResolvedValue(undefined)
})

describe('independent resource saving', () => {
  it('reuses committed tag and upload IDs after a later upload fails', async () => {
    const { state, editor } = session()
    vi.mocked(api.createAttachment)
      .mockResolvedValueOnce('first-id')
      .mockRejectedValueOnce(new Error('upload failed'))
    await expect(editor.save(draft())).rejects.toThrow('upload failed')
    expect(state.stage).toBe('attachments')
    expect(api.createQuestion).not.toHaveBeenCalled()
    // The page reconstructs its draft on retry; the editor retains checkpoints.
    await editor.save(draft())
    expect(api.createTag).toHaveBeenCalledTimes(1)
    expect(api.createAttachment).toHaveBeenCalledTimes(3)
    expect(api.createQuestion).toHaveBeenCalledWith(
      expect.objectContaining({
        attachmentIds: ['first-id', 'second-id'],
        tagIds: ['tag']
      })
    )
  })

  it('retains uploads after the question fails, without deleting resources', async () => {
    const { editor } = session()
    vi.mocked(api.createQuestion).mockRejectedValueOnce(
      new Error('question failed')
    )
    await expect(editor.save(draft())).rejects.toThrow('question failed')
    await editor.save(draft())
    expect(api.createAttachment).toHaveBeenCalledTimes(2)
    expect(api.createTag).toHaveBeenCalledTimes(1)
    expect(api.deleteAttachment).not.toHaveBeenCalled()
  })

  it('sends complete replacement fields, including null and empty relations', async () => {
    const { editor } = session()
    await editor.save({
      ...draft(),
      id: 'question',
      attachments: [],
      tags: [],
      stem: '  文本  '
    })
    expect(api.updateQuestion).toHaveBeenCalledWith({
      id: 'question',
      sourceId: null,
      questionType: null,
      stem: '  文本  ',
      correctAnswer: '',
      explanation: null,
      note: null,
      tagIds: [],
      attachmentIds: []
    })
  })

  it('keeps a committed question successful when cleanup fails and retries cleanup alone', async () => {
    const { state, editor } = session()
    vi.mocked(api.deleteAttachment).mockRejectedValueOnce({
      code: 'STORAGE_ERROR',
      message: 'busy'
    })
    const result = await editor.save({ ...draft(), id: 'question' }, ['old'])
    expect(result.id).toBe('question')
    expect(state.committed?.id).toBe('question')
    expect(state.cleanupIds).toEqual(['old'])
    await editor.retryCleanup()
    expect(state.cleanupIds).toEqual([])
    expect(state.stage).toBe('saved')
    expect(api.updateQuestion).toHaveBeenCalledTimes(1)
  })

  it('preserves shared attachments and treats already removed resources as complete', async () => {
    const { state, editor } = session()
    vi.mocked(api.deleteAttachment)
      .mockRejectedValueOnce({ code: 'RESOURCE_IN_USE' })
      .mockRejectedValueOnce({ code: 'NOT_FOUND' })
    await editor.save(
      {
        ...draft(),
        id: 'question',
        attachments: [{ id: 'retained', base64Data: 'same' }]
      },
      ['retained', 'shared', 'missing']
    )
    expect(api.deleteAttachment).not.toHaveBeenCalledWith('retained')
    expect(state.cleanupIds).toEqual([])
    expect(api.createAttachment).not.toHaveBeenCalled()
  })

  it('rejects concurrent saves and checkpoints source materialization before a failure', async () => {
    const { editor } = session()
    let resolve!: (value: {
      selection: { kind: 'existing'; sourceId: string }
      sourceId: string
    }) => void
    vi.mocked(materializeSourceSelection).mockReturnValueOnce(
      new Promise((done) => {
        resolve = done
      })
    )
    const input = draft()
    const pending = editor.save(input)
    await expect(editor.save(draft())).rejects.toThrow('保存正在进行')
    vi.mocked(api.listTags).mockRejectedValueOnce(new Error('tags unavailable'))
    resolve({
      selection: { kind: 'existing', sourceId: 'source' },
      sourceId: 'source'
    })
    await expect(pending).rejects.toThrow('tags unavailable')
    expect(input.source).toEqual({ kind: 'existing', sourceId: 'source' })
  })

  it('does not create a second question when retrying an already committed draft', async () => {
    const { editor } = session()
    const input = draft()
    await editor.save(input)
    await editor.save(input)
    expect(api.createQuestion).toHaveBeenCalledTimes(1)
    expect(api.updateQuestion).toHaveBeenCalledTimes(1)
  })
})
