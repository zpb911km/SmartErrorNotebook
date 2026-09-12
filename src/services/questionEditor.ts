import * as api from '../api'
import type { Question, UpdateQuestionRequest } from '../types'
import type { SourceSelection } from './sourceSelection'
import { materializeSourceSelection } from './sourcePersistence'
import { prepareAttachment } from '../utils/attachments'

export interface TagDraft {
  id?: string
  name: string
  color: string
}
export interface AttachmentDraft {
  id?: string
  base64Data: string
  mimeType?: string
}
export interface QuestionDraft {
  id?: string
  source: SourceSelection
  questionType: Question['questionType']
  stem: string
  correctAnswer: string
  explanation: string | null
  note: string | null
  tags: TagDraft[]
  attachments: AttachmentDraft[]
}
export type SaveStage =
  'idle' | 'source' | 'tags' | 'attachments' | 'question' | 'cleanup' | 'saved'
export interface SaveState {
  stage: SaveStage
  committed: Question | null
  cleanupIds: string[]
  error: unknown
}

/** One editor session owns resource checkpoints; no rollback across resources. */
export function createQuestionEditor(state: SaveState) {
  const uploads = new Map<string, string>()
  const tags = new Map<string, string>()
  let running = false

  async function retryCleanup() {
    state.stage = 'cleanup'
    state.error = null
    for (const id of [...state.cleanupIds]) {
      try {
        await api.deleteAttachment(id)
      } catch (error) {
        const code =
          typeof error === 'object' && error && 'code' in error
            ? error.code
            : null
        if (code !== 'NOT_FOUND' && code !== 'RESOURCE_IN_USE') {
          state.error = error
          continue
        }
      }
      state.cleanupIds = state.cleanupIds.filter((value) => value !== id)
    }
    state.stage = state.cleanupIds.length ? 'cleanup' : 'saved'
  }

  async function save(
    draft: QuestionDraft,
    originalAttachmentIds: string[] = []
  ) {
    if (running) throw new Error('保存正在进行')
    running = true
    state.error = null
    state.committed = null
    try {
      state.stage = 'source'
      const source = await materializeSourceSelection(draft.source)
      draft.source = source.selection
      state.stage = 'tags'
      const known = await api.listTags()
      const tagIds: string[] = []
      for (const tag of draft.tags) {
        const key = JSON.stringify([tag.name, tag.color])
        let id =
          tag.id ??
          known.find(
            (item) => item.name === tag.name && item.color === tag.color
          )?.id ??
          tags.get(key)
        if (!id) {
          id = (await api.createTag({ name: tag.name, color: tag.color })).id
          tags.set(key, id)
        }
        tag.id = id
        tagIds.push(id)
      }
      state.stage = 'attachments'
      const attachmentIds: string[] = []
      for (const attachment of draft.attachments) {
        let id = attachment.id ?? uploads.get(attachment.base64Data)
        if (!id) {
          id = await api.createAttachment(
            await prepareAttachment(attachment.base64Data)
          )
          uploads.set(attachment.base64Data, id)
        }
        attachment.id = id
        attachmentIds.push(id)
      }
      const request: Omit<UpdateQuestionRequest, 'id'> = {
        sourceId: source.sourceId,
        questionType: draft.questionType,
        stem: draft.stem,
        correctAnswer: draft.correctAnswer,
        explanation: draft.explanation,
        note: draft.note,
        tagIds: [...new Set(tagIds)],
        attachmentIds: [...new Set(attachmentIds)]
      }
      state.stage = 'question'
      const question = draft.id
        ? await api.updateQuestion({ id: draft.id, ...request })
        : await api.createQuestion(request)
      // Checkpoint the question before any subsequent operation can fail.
      draft.id = question.id
      state.committed = question
      state.cleanupIds = [
        ...new Set([
          ...state.cleanupIds,
          ...originalAttachmentIds.filter(
            (id) => !question.attachmentIds.includes(id)
          )
        ])
      ]
      await retryCleanup()
      return question
    } catch (error) {
      state.error = error
      throw error
    } finally {
      running = false
    }
  }

  function reset() {
    uploads.clear()
    tags.clear()
    state.stage = 'idle'
    state.committed = null
    state.error = null
    // Pending cleanup survives closing an editor and is explicitly retryable.
  }
  return { save, retryCleanup, reset }
}
