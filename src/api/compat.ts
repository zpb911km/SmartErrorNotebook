import type {
  Attachment as LegacyAttachment,
  CreateAttachmentInput,
  ErrorQuestion,
  ErrorTags,
  QuestionFilter as LegacyQuestionFilter,
  SRSData,
  Source as LegacySource,
  Subject as LegacySubject,
  UpdateQuestionInput
} from '../types/legacy'
import type {
  Attachment,
  Question,
  QuestionType,
  Source,
  SrsData,
  Subject,
  Tag
} from '../types'
import {
  createAttachment,
  createQuestion,
  createTag,
  deleteAttachment,
  deleteQuestion,
  getLibraryStatistics,
  getAttachment,
  getQuestion,
  getSrsData,
  getSource,
  listQuestions,
  listSources,
  listSrsData,
  listSubjects,
  listTags,
  createSource,
  deleteSource,
  resetReviewProgress,
  submitReview,
  updateQuestion,
  updateSource,
  updateSubject,
  updateTag,
  createSubject,
  deleteSubject
} from './index'
import { compressImageIfTooLarge } from '../utils/imageCompression'

export {
  blobUrlToBase64,
  buildDataUrl,
  fileToBase64
} from './legacy/attachments'

const toSeconds = (value: string | null) =>
  value ? Math.floor(new Date(value).getTime() / 1000) : null

const typeToWire = (value?: string): QuestionType | null => {
  const types: Record<string, QuestionType> = {
    单选题: 'SINGLE_SELECT',
    多选题: 'MULTIPLE_SELECT',
    判断题: 'TRUE_FALSE',
    填空题: 'FILL_IN_THE_BLANK',
    简答题: 'SHORT_ANSWER',
    计算题: 'CALCULATION',
    论述题: 'ESSAY'
  }
  return value ? (types[value] ?? (value as QuestionType)) : null
}

const typeFromWire = (value: QuestionType | null) =>
  value
    ? {
        SINGLE_SELECT: '单选题',
        MULTIPLE_SELECT: '多选题',
        TRUE_FALSE: '判断题',
        FILL_IN_THE_BLANK: '填空题',
        SHORT_ANSWER: '简答题',
        CALCULATION: '计算题',
        ESSAY: '论述题'
      }[value]
    : ''

const toLegacySrs = (value: SrsData): SRSData & { is_due: boolean } => ({
  id: value.questionId,
  question_id: value.questionId,
  stability: value.stability,
  difficulty: value.difficulty,
  recall_rate: value.retrievability,
  next_review_at: toSeconds(value.nextReviewAt),
  last_review_at: toSeconds(value.lastReviewAt),
  review_count: value.reviewCount,
  feedback_history: '[]',
  is_due: value.isDue
})

type HydratedQuestionData = Question & {
  subject: Subject | null
  source: Source | null
  tags: Tag[]
  srs: SrsData | null
}

const toLegacyQuestion = (
  value: Question & Partial<Pick<HydratedQuestionData, 'subject' | 'source' | 'tags' | 'srs'>>,
  fallbackSubjectId = ''
): ErrorQuestion & any => {
  const subjectId = value.subject?.id ?? fallbackSubjectId
  const sourceId = value.source?.id ?? value.sourceId ?? undefined
  const type = typeFromWire(value.questionType)
  return {
    id: value.id,
    user_id: '',
    userid: '',
    subject_id: subjectId,
    subjectid: subjectId,
    source_id: sourceId,
    sourceid: sourceId,
    prompt: value.stem,
    type,
    type_: type,
    answer: value.correctAnswer || undefined,
    analysis: value.explanation ?? undefined,
    error_note: value.note ?? undefined,
    created_at: toSeconds(value.createdAt),
    updated_at: toSeconds(value.updatedAt),
    tags: value.tags ?? [],
    srs: value.srs ? toLegacySrs(value.srs) : null
  }
}

const toLegacySource = (value: Source): LegacySource & any => ({
  id: value.id,
  subject_id: value.subjectId ?? undefined,
  question_id: undefined,
  book: value.book ?? undefined,
  chapter: value.chapter ?? undefined,
  knowledge: value.knowledge ?? undefined,
  created_at: toSeconds(value.createdAt),
  updated_at: toSeconds(value.updatedAt)
})

const toLegacyTag = (value: Tag, questionId = ''): ErrorTags => ({
  id: value.id,
  question_id: questionId,
  name: value.name,
  color: value.color
})

const toLegacyAttachment = (
  value: Attachment,
  questionId: string
): LegacyAttachment & any => ({
  id: value.id,
  question_id: questionId,
  type: 'original',
  type_: 'original',
  file_type: value.mimeType.split('/')[1] || 'image',
  base64_data: value.base64Data,
  hash: value.sha256
})

const attachmentMime = (input: CreateAttachmentInput) => {
  const data = input.base64_data.includes(',')
    ? input.base64_data.split(',')[1]
    : input.base64_data
  if (data.startsWith('/9j/')) return 'image/jpeg'
  if (data.startsWith('iVBOR')) return 'image/png'
  if (data.startsWith('UklGR')) return 'image/webp'
  if (data.startsWith('R0lGOD')) return 'image/gif'
  if (input.file_type === 'jpg') return 'image/jpeg'
  return input.file_type.includes('/')
    ? input.file_type
    : `image/${input.file_type === 'image' ? 'png' : input.file_type}`
}

const attachmentRequests = async (attachments: CreateAttachmentInput[]) =>
  Promise.all(
    attachments.map(async (item) => {
      const base64Data = await compressImageIfTooLarge(item.base64_data)
      return {
        mimeType: attachmentMime({ ...item, base64_data: base64Data }),
        base64Data
      }
    })
  )

// Migration adapter only: the resources composed here have independent
// lifecycles. A successful create is never rolled back because a later IPC call
// fails; future UI code should invoke each Current API explicitly instead.
export class CompatibilityOperationError extends Error {
  constructor(
    message: string,
    readonly committed: boolean,
    readonly primaryError: unknown,
    readonly cleanupErrors: unknown[]
  ) {
    super(message)
    this.name = 'CompatibilityOperationError'
  }
}

const apiErrorCode = (error: unknown) =>
  typeof error === 'object' && error !== null && 'code' in error
    ? String(error.code)
    : null

const isBenignCleanupError = (error: unknown) =>
  ['NOT_FOUND', 'RESOURCE_IN_USE'].includes(apiErrorCode(error) ?? '')

const cleanup = async (action: () => Promise<unknown>) => {
  try {
    await action()
    return null
  } catch (error) {
    return isBenignCleanupError(error) ? null : error
  }
}

const writeRequest = async (
  question: Question,
  patch: UpdateQuestionInput
) => {
  let sourceId = question.sourceId
  if (patch.subject_id !== undefined) {
    sourceId = await resolveQuestionSourceId(
      patch.subject_id,
      patch.source_id !== undefined ? patch.source_id : question.sourceId
    )
  } else if (patch.source_id !== undefined) {
    sourceId = patch.source_id || null
  }
  return {
    id: question.id,
    sourceId,
    questionType: typeToWire(patch.type ?? question.questionType ?? undefined),
    stem: patch.prompt ?? question.stem,
    correctAnswer: patch.answer ?? question.correctAnswer,
    explanation: patch.analysis ?? question.explanation,
    note: patch.error_note ?? question.note,
    tagIds: question.tagIds,
    attachmentIds: question.attachmentIds
  }
}

const resolveTagIds = async (
  values: Array<{ name: string; color: string }>
) => {
  const known = await listTags()
  const ids: string[] = []
  for (const value of values) {
    let tag = known.find(
      (item) => item.name === value.name && item.color === value.color
    )
    if (!tag) {
      tag = await createTag(value)
      known.push(tag)
    }
    ids.push(tag.id)
  }
  return {
    ids: [...new Set(ids)],
    tagsById: new Map(known.map((tag) => [tag.id, tag]))
  }
}

const createAttachments = async (inputs: CreateAttachmentInput[]) => {
  const attachments: Attachment[] = []
  for (const request of await attachmentRequests(inputs)) {
    const id = await createAttachment(request)
    attachments.push(await getAttachment(id))
  }
  return attachments
}

const cleanupAttachments = async (ids: string[]) => {
  const errors: unknown[] = []
  for (const id of ids) {
    const error = await cleanup(() => deleteAttachment(id))
    if (error) errors.push(error)
  }
  return errors
}

const hydrateQuestions = async (
  questions: Question[],
  includeSrs = true
): Promise<HydratedQuestionData[]> => {
  if (questions.length === 0) return []
  const at = new Date().toISOString()
  const [sources, subjects, tags, srsItems] = await Promise.all([
    listSources(),
    listSubjects(),
    listTags(),
    includeSrs ? listSrsData(at) : Promise.resolve([])
  ])
  const sourceById = new Map(sources.map((source) => [source.id, source]))
  const subjectById = new Map(subjects.map((subject) => [subject.id, subject]))
  const tagById = new Map(tags.map((tag) => [tag.id, tag]))
  const srsByQuestionId = new Map(srsItems.map((srs) => [srs.questionId, srs]))
  return questions.map((question) => {
    const source = question.sourceId
      ? (sourceById.get(question.sourceId) ?? null)
      : null
    return {
      ...question,
      source,
      subject: source?.subjectId
        ? (subjectById.get(source.subjectId) ?? null)
        : null,
      tags: question.tagIds.flatMap((id) => {
        const tag = tagById.get(id)
        return tag ? [tag] : []
      }),
      srs: srsByQuestionId.get(question.id) ?? null
    }
  })
}

const hydrateQuestion = async (question: Question) =>
  (await hydrateQuestions([question], false))[0]

export const getSubjects = async (): Promise<LegacySubject[]> =>
  (await listSubjects()) as LegacySubject[]

export const addSubject = async (
  input: Omit<LegacySubject, 'id'>
): Promise<LegacySubject> =>
  (await createSubject({
    name: input.name,
    color: input.color ?? ''
  })) as LegacySubject

export const editSubject = async (input: LegacySubject) =>
  (await updateSubject({
    id: input.id,
    name: input.name,
    color: input.color ?? ''
  })) as LegacySubject

export const removeSubject = deleteSubject

export const getSources = async (subjectId?: string): Promise<LegacySource[]> =>
  (await listSources(subjectId)).map(toLegacySource)

export const getSourceForUi = async (id: string) =>
  toLegacySource(await getSource(id))

const sortedUnique = (values: Array<string | null>) =>
  [...new Set(values.filter((value): value is string => value !== null))].sort()

export const getBooks = async (subjectId?: string) =>
  sortedUnique((await listSources(subjectId)).map((source) => source.book))

export const getChapters = async (book: string, subjectId?: string) =>
  sortedUnique(
    (await listSources(subjectId))
      .filter((source) => source.book === book)
      .map((source) => source.chapter)
  )

export const getKnowledges = async (
  book: string,
  chapter: string,
  subjectId?: string
) =>
  sortedUnique(
    (await listSources(subjectId))
      .filter((source) => source.book === book && source.chapter === chapter)
      .map((source) => source.knowledge)
  )

const pendingSourceResolutions = new Map<string, Promise<string>>()

const resolveSourceId = (input: {
  subject_id?: string
  book?: string
  chapter?: string
  knowledge?: string
}) => {
  const request = {
    subjectId: input.subject_id ?? null,
    book: input.book ?? null,
    chapter: input.chapter ?? null,
    knowledge: input.knowledge ?? null
  }
  const key = JSON.stringify([
    request.subjectId, request.book, request.chapter, request.knowledge
  ])
  const pending = pendingSourceResolutions.get(key)
  if (pending) return pending

  const resolution = (async () => {
    const sources = await listSources(request.subjectId ?? undefined)
    const existing = sources.find(
      (source) =>
        source.subjectId === request.subjectId &&
        source.book === request.book &&
        source.chapter === request.chapter &&
        source.knowledge === request.knowledge
    )
    return existing ? existing.id : (await createSource(request)).id
  })().finally(() => {
    pendingSourceResolutions.delete(key)
  })
  pendingSourceResolutions.set(key, resolution)
  return resolution
}

// Question only persists sourceId. During the legacy-UI migration, subjectId is
// a projection of Source and must never be allowed to disagree with it.
const resolveQuestionSourceId = async (
  subjectId: string,
  requestedSourceId?: string | null
) => {
  if (requestedSourceId) {
    const requested = await getSource(requestedSourceId)
    if (requested.subjectId === (subjectId || null)) return requested.id
    if (!subjectId) return null
    return resolveSourceId({
      subject_id: subjectId,
      book: requested.book ?? undefined,
      chapter: requested.chapter ?? undefined,
      knowledge: requested.knowledge ?? undefined
    })
  }
  if (!subjectId) return null
  return resolveSourceId({ subject_id: subjectId })
}

export const getOrCreateSourceId = (input: {
  subject_id?: string
  book?: string
  chapter?: string
  knowledge?: string
}) => resolveSourceId(input)

export const editSource = async (input: {
  id: string
  subject_id?: string
  book?: string
  chapter?: string
  knowledge?: string
}) => {
  const current = await getSource(input.id)
  return toLegacySource(
    await updateSource({
      id: input.id,
      subjectId: input.subject_id ?? current.subjectId,
      book: input.book ?? current.book,
      chapter: input.chapter ?? current.chapter,
      knowledge: input.knowledge ?? current.knowledge
    })
  )
}

export const removeSource = deleteSource

export const getQuestions = async (
  filter?: LegacyQuestionFilter
): Promise<ErrorQuestion[]> => {
  const filterBySubject = Boolean(filter?.subject_id)
  const page = await listQuestions({
    filter: { search: filter?.search },
    sort: ['UPDATED_AT_DESC'],
    limit: filterBySubject ? undefined : filter?.limit,
    offset: filterBySubject ? undefined : filter?.offset
  })
  const hydrated = await hydrateQuestions(page.items)
  const items = filterBySubject
    ? hydrated.filter((question) => question.subject?.id === filter?.subject_id)
    : hydrated
  const offset = filterBySubject ? (filter?.offset ?? 0) : 0
  const end = filterBySubject
    ? filter?.limit === undefined
      ? items.length
      : offset + filter.limit
    : items.length
  return items.slice(offset, end).map((question) => toLegacyQuestion(question))
}

export const addQuestion = async (
  input: Omit<ErrorQuestion, 'id'>,
  tags: Array<{ name: string; color: string }> = [],
  attachments: CreateAttachmentInput[] = []
): Promise<ErrorQuestion> => {
  const sourceId = await resolveQuestionSourceId(
    input.subject_id,
    input.source_id
  )
  const resolvedTags = await resolveTagIds(tags)
  const createdAttachments = await createAttachments(attachments)
  const question = await createQuestion({
    sourceId,
    questionType: typeToWire(String(input.type)),
    stem: input.prompt,
    correctAnswer: input.answer ?? '',
    explanation: input.analysis ?? null,
    note: input.error_note ?? null,
    tagIds: resolvedTags.ids,
    attachmentIds: createdAttachments.map((item) => item.id)
  })
  return toLegacyQuestion(question, input.subject_id)
}

export const getQuestionForUi = async (id: string) =>
  toLegacyQuestion(await hydrateQuestion(await getQuestion(id)))

export const editQuestion = async (input: UpdateQuestionInput) => {
  const current = await getQuestion(input.id)
  const updated = await updateQuestion(await writeRequest(current, input))
  return toLegacyQuestion(updated, input.subject_id)
}

export const saveQuestionAggregate = async (
  input: UpdateQuestionInput,
  tags: Array<{ name: string; color: string }>,
  currentAttachments: Array<LegacyAttachment & { type_?: string }>,
  originalAttachments: Array<LegacyAttachment & { type_?: string }>
) => {
  // Transitional adapter for the legacy aggregate-shaped UI. Source, tag,
  // attachment and question records deliberately have independent lifecycles;
  // do not turn this into a multi-resource Current IPC command or imply atomicity.
  // Future UI should save each resource through its Current API and track its
  // committed/error/retry state independently.
  const current = await getQuestion(input.id)
  const retainedIds: string[] = []
  const uploads: CreateAttachmentInput[] = []

  for (const item of currentAttachments) {
    const original = originalAttachments.find((value) => value.id === item.id)
    if (
      !item.id.startsWith('temp-') &&
      original?.base64_data === item.base64_data
    ) {
      retainedIds.push(item.id)
    } else {
      uploads.push({
        question_id: input.id,
        type_: item.type_ ?? item.type ?? 'original',
        file_type: item.file_type,
        base64_data: item.base64_data
      })
    }
  }

  const resolvedTags = await resolveTagIds(tags)
  const newAttachments = await createAttachments(uploads)
  const updated = await updateQuestion({
    ...(await writeRequest(current, input)),
    tagIds: resolvedTags.ids,
    attachmentIds: [...retainedIds, ...newAttachments.map((item) => item.id)]
  })
  const removedAttachmentIds = originalAttachments
    .map((item) => item.id)
    .filter((id) => !id.startsWith('temp-') && !retainedIds.includes(id))
  const cleanupErrors = await cleanupAttachments(removedAttachmentIds)
  // Cleanup happens after the question has committed its new attachment IDs.
  // Keep reporting partial completion here; future per-resource UI should expose
  // cleanup as independently retryable work rather than rolling back the question.
  if (cleanupErrors.length > 0) {
    throw new CompatibilityOperationError(
      '题目已保存，但旧附件清理失败',
      true,
      null,
      cleanupErrors
    )
  }
  return toLegacyQuestion(updated, input.subject_id)
}

export const removeQuestion = deleteQuestion
export const getQuestionStats = async () => ({
  total: (await getLibraryStatistics(new Date().toISOString())).questionTotal
})

export const getErrorTags = async () =>
  (await listTags()).map((tag) => toLegacyTag(tag))

export const getFullErrorTags = async () => {
  const [questions, tags] = await Promise.all([
    listQuestions({ sort: [] }).then((response) => response.items),
    listTags()
  ])
  const tagById = new Map(tags.map((tag) => [tag.id, tag]))
  return questions.flatMap((question) =>
    question.tagIds.flatMap((tagId) => {
      const tag = tagById.get(tagId)
      return tag ? [toLegacyTag(tag, question.id)] : []
    })
  )
}

export const getErrorTagsForQuestion = async (questionId: string) => {
  const [question, tags] = await Promise.all([
    getQuestion(questionId),
    listTags()
  ])
  const ids = new Set(question.tagIds)
  return tags
    .filter((tag) => ids.has(tag.id))
    .map((tag) => toLegacyTag(tag, questionId))
}

export const addErrorTagsForQuestion = async (
  questionId: string,
  tags: Array<{ name: string; color: string }>
) => {
  const question = await getQuestion(questionId)
  const resolvedTags = await resolveTagIds(tags)
  const updated = await updateQuestion({
    ...(await writeRequest(question, { id: questionId })),
    tagIds: [...new Set([...question.tagIds, ...resolvedTags.ids])]
  })
  const tagsById = resolvedTags.tagsById
  return updated.tagIds.flatMap((tagId) => {
    const tag = tagsById.get(tagId)
    return tag ? [toLegacyTag(tag, questionId)] : []
  })
}

export const removeErrorTagFromQuestion = async (
  tagId: string,
  questionId: string
) => {
  const question = await getQuestion(questionId)
  await updateQuestion({
    ...(await writeRequest(question, { id: questionId })),
    tagIds: question.tagIds.filter((id) => id !== tagId)
  })
}

export const getAttachmentsForQuestion = async (questionId: string) =>
  Promise.all(
    (await getQuestion(questionId)).attachmentIds.map(getAttachment)
  ).then((attachments) =>
    attachments.map((item) => toLegacyAttachment(item, questionId))
  )

export const addAttachmentsForQuestion = async (
  questionId: string,
  attachments: CreateAttachmentInput[]
) => {
  const question = await getQuestion(questionId)
  const existingAttachments = await Promise.all(
    question.attachmentIds.map(getAttachment)
  )
  const createdAttachments = await createAttachments(attachments)
  await updateQuestion({
    ...(await writeRequest(question, { id: questionId })),
    attachmentIds: [
      ...question.attachmentIds,
      ...createdAttachments.map((item) => item.id)
    ]
  })
  return [...existingAttachments, ...createdAttachments].map((item) =>
    toLegacyAttachment(item, questionId)
  )
}

export const getQuestionSrs = async (questionId: string) => {
  try {
    return toLegacySrs(await getSrsData(questionId, new Date().toISOString()))
  } catch (error) {
    if (
      typeof error === 'object' &&
      error !== null &&
      'code' in error &&
      error.code === 'NOT_FOUND'
    ) {
      return null
    }
    throw error
  }
}

export const ensureQuestionSrs = async (questionId: string) => {
  const existing = await getQuestionSrs(questionId)
  if (existing) return existing
  return toLegacySrs(
    await resetReviewProgress(questionId, new Date().toISOString())
  )
}

export const getAllSrs = async () =>
  (await listSrsData(new Date().toISOString())).map(toLegacySrs)

export const getDueCount = async () =>
  (await getLibraryStatistics(new Date().toISOString())).dueCount

export const getSrsStatistics = async () => {
  const value = await getLibraryStatistics(new Date().toISOString())
  return {
    total: value.cardTotal,
    due_count: value.dueCount,
    new_cards: value.newCardCount,
    avg_stability: value.averageStability,
    avg_difficulty: value.averageDifficulty,
    total_reviews: value.totalReviews
  }
}

export const submitReviewResult = async (input: {
  question_id: string
  feedback: number
}) => {
  const value = await submitReview({
    questionId: input.question_id,
    feedback: input.feedback,
    reviewedAt: new Date().toISOString()
  })
  return {
    next_interval_days: value.nextIntervalDays,
    new_stability: value.srs.stability,
    new_difficulty: value.srs.difficulty,
    next_review_at: toSeconds(value.srs.nextReviewAt) ?? 0
  }
}

export interface ReviewOutput {
  next_interval_days: number
  new_stability: number
  new_difficulty: number
  next_review_at: number
}

export const editTag = (id: string, name: string, color: string) =>
  updateTag({ id, name, color })
