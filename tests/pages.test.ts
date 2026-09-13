// @vitest-environment happy-dom
import { invoke } from '@tauri-apps/api/core'
import { Quasar } from 'quasar'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { type Component, createApp, nextTick } from 'vue'

import ImportModal from '../src/components/ImportModal.vue'
import { quasarOptions } from '../src/quasar'
import { llm } from '../src/services/llm'
import { clearReviewQueue, setReviewQueue } from '../src/services/reviewStore'
import type { Question, SrsData } from '../src/types'
import { inquiryAIAddInfo, type TaggedResult } from '../src/utils/inquiry'
import Add from '../src/views/Add.vue'
import Manage from '../src/views/Manage.vue'
import Detail from '../src/views/Manage-Detail.vue'
import Preview from '../src/views/Preview.vue'
import Profile from '../src/views/Profile.vue'
import Review from '../src/views/Review-Detail.vue'

vi.mock('../src/utils/inquiry', () => ({ inquiryAIAddInfo: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('../src/utils/dialog', () => ({
  confirmAction: vi.fn(async () => true),
  showAlert: vi.fn()
}))
vi.mock('../src/components/Icon.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/MarkdownTextarea.vue', () => ({
  default: {
    props: ['modelValue'],
    setup: (props: { modelValue: string }) => () => props.modelValue
  }
}))
vi.mock('../src/components/CameraModal.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/ImageEditor.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/ImagePreview.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/SubjectSelector.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/SourceSelector.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/ErrorTagSelector.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/components/ExportModal.vue', () => ({
  default: { render: () => null }
}))
vi.mock('../src/utils/notification', () => ({
  showInfo: vi.fn(),
  showError: vi.fn(),
  showSuccess: vi.fn()
}))
const navigation = vi.hoisted(() => ({
  push: vi.fn(),
  replace: vi.fn(),
  back: vi.fn()
}))
vi.mock('vue-router', () => ({
  useRouter: () => navigation,
  useRoute: () => ({ params: { id: 'question' }, query: {}, path: '/manage' })
}))

const at = '2026-09-11T00:00:00Z'
const question: Question = {
  id: 'question',
  stem: 'PAGE_STEM',
  correctAnswer: 'PAGE_ANSWER',
  explanation: 'PAGE_EXPLANATION',
  note: 'PAGE_NOTE',
  questionType: 'SHORT_ANSWER',
  sourceId: 'source',
  tagIds: ['tag'],
  attachmentIds: [],
  createdAt: at,
  updatedAt: at
}
const srs: SrsData = {
  questionId: 'question',
  stability: 4,
  difficulty: 5,
  retrievability: 0.5,
  reviewCount: 2,
  nextReviewAt: at,
  lastReviewAt: at,
  isDue: true
}
let savedQuestions: Question[]
let reviewCards: SrsData[]
let failCreate: boolean
const failures: unknown[] = []
const dispose: Array<() => void> = []

function mount<T>(component: Component, props: Record<string, unknown> = {}) {
  const element = document.createElement('div')
  document.body.append(element)
  const app = createApp(component, props)
  app.use(Quasar, quasarOptions)
  app.component('Icon', { render: () => null })
  app.component('MarkdownTextarea', {
    props: ['modelValue'],
    setup: (props) => () => props.modelValue
  })
  app.directive('ripple', {})
  app.directive('scroll-reveal', {})
  app.config.errorHandler = (error) => failures.push(error)
  const instance = app.mount(element)
  dispose.push(() => {
    app.unmount()
    element.remove()
  })
  return { element, state: instance.$.setupState as T }
}

beforeEach(() => {
  vi.clearAllMocks()
  failures.length = 0
  savedQuestions = [{ ...question }]
  reviewCards = [{ ...srs }]
  failCreate = false
  vi.spyOn(console, 'log').mockImplementation(() => {})
  vi.spyOn(console, 'warn').mockImplementation(() => {})
  vi.spyOn(console, 'error').mockImplementation(() => {})
  vi.stubGlobal('alert', vi.fn())
  vi.stubGlobal(
    'confirm',
    vi.fn(() => true)
  )
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const request =
      (args as { request?: Record<string, unknown> })?.request ?? {}
    switch (command) {
      case 'list_questions':
        return { items: savedQuestions, total: savedQuestions.length }
      case 'get_question':
        return { question: savedQuestions[0] }
      case 'list_subjects':
        return {
          subjects: [
            {
              id: 'subject',
              name: '数学',
              color: '#123456',
              createdAt: at,
              updatedAt: at
            }
          ]
        }
      case 'list_sources':
        return {
          sources: [
            {
              id: 'source',
              subjectId: 'subject',
              book: null,
              chapter: null,
              knowledge: null,
              createdAt: at,
              updatedAt: at
            }
          ]
        }
      case 'list_tags':
        return { tags: [{ id: 'tag', name: '计算', color: '#123456' }] }
      case 'list_srs_data':
        return { items: reviewCards }
      case 'get_library_statistics':
        return {
          statistics: {
            questionTotal: 1,
            cardTotal: 1,
            dueCount: 1,
            newCardCount: 0,
            averageStability: 4,
            averageDifficulty: 5,
            totalReviews: 2
          }
        }
      case 'create_question': {
        if (failCreate) throw { code: 'STORAGE_ERROR', message: '不可写入' }
        const created = { ...question, ...request, id: 'created' } as Question
        savedQuestions.push(created)
        return { question: created }
      }
      case 'update_question': {
        savedQuestions[0] = { ...question, ...request } as Question
        return { question: savedQuestions[0] }
      }
      case 'delete_question':
        savedQuestions = []
        return { id: request.id }
      case 'submit_review':
        return { srs: { ...srs, reviewCount: 3 }, nextIntervalDays: 7 }
      default:
        throw new Error(`Unexpected IPC: ${command}`)
    }
  })
})

afterEach(() => {
  dispose
    .splice(0)
    .reverse()
    .forEach((cleanup) => cleanup())
  clearReviewQueue()
  expect(failures).toEqual([])
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

it('renders management and review lists from Current response fields without write commands', async () => {
  const manage = mount(Manage)
  const preview = mount(Preview)
  await vi.waitFor(() => {
    expect(manage.element.textContent).toContain('PAGE_STEM')
    expect(preview.element.textContent).toContain('PAGE_STEM')
    expect(manage.element.textContent).toContain('数学')
  })
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(([command]) => command.startsWith('list_'))
  ).toBe(true)
})

it('keeps unscheduled dates null and places them after scheduled review cards', async () => {
  savedQuestions = [
    { ...question, id: 'unscheduled' },
    { ...question, id: 'scheduled' }
  ]
  reviewCards = [
    {
      ...srs,
      questionId: 'unscheduled',
      isDue: false,
      nextReviewAt: null,
      lastReviewAt: null
    },
    { ...srs, questionId: 'scheduled', isDue: false }
  ]
  const { state } = mount<{
    notDueList: Array<{ id: string; nextReviewAt: number | null }>
  }>(Preview)
  await vi.waitFor(() => expect(state.notDueList).toHaveLength(2))
  expect(state.notDueList.map((card) => card.id)).toEqual([
    'scheduled',
    'unscheduled'
  ])
  expect(state.notDueList[1].nextReviewAt).toBeNull()
})

it('adds a question using complete Current fields from the existing form', async () => {
  const { element, state } = mount<{
    form: {
      prompt: string
      type: string
      answer: string
      analysis: string
      note: string
    }
  }>(Add)
  Object.assign(state.form, {
    prompt: 'NEW_STEM',
    type: '简答题',
    answer: 'NEW_ANSWER',
    analysis: 'NEW_EXPLANATION',
    note: 'NEW_NOTE'
  })
  await nextTick()
  element.querySelector<HTMLButtonElement>('button.save')!.click()
  await vi.waitFor(() => expect(savedQuestions).toHaveLength(2))
  expect(savedQuestions[1]).toMatchObject({
    stem: 'NEW_STEM',
    questionType: 'SHORT_ANSWER',
    sourceId: null,
    correctAnswer: 'NEW_ANSWER',
    explanation: 'NEW_EXPLANATION',
    note: 'NEW_NOTE',
    tagIds: [],
    attachmentIds: []
  })
})

it('preserves manual edits while AI results arrive and ignores duplicate recognition', async () => {
  vi.spyOn(llm, 'isConfigured').mockReturnValue(true)
  let resolve!: (results: TaggedResult[]) => void
  const pending = new Promise<TaggedResult[]>((done) => {
    resolve = done
  })
  vi.mocked(inquiryAIAddInfo).mockReturnValue(pending)
  const { state } = mount<{
    form: { prompt: string; answer: string }
    imageUrls: string[]
    inquiryAI: () => Promise<void>
  }>(Add)
  state.imageUrls.push('data:image/png;base64,fixture')
  state.form.prompt = 'Original'
  const recognition = state.inquiryAI()
  await state.inquiryAI()
  expect(inquiryAIAddInfo).toHaveBeenCalledTimes(5)
  state.form.prompt = 'Manual edit'
  resolve([
    {
      tag: 'question_text',
      content: 'AI text',
      success: true,
      parsedContent: 'AI text'
    }
  ])
  await recognition
  expect(state.form.prompt).toBe('Manual edit')
  expect(state.form.answer).toBe('AI text')
})

it.each(['question_text', 'question_type'])(
  'tracks independent AI spinners when %s finishes first',
  async (first) => {
    vi.spyOn(llm, 'isConfigured').mockReturnValue(true)
    const pending = new Map<
      string,
      {
        resolve: (value: TaggedResult[]) => void
        reject: (error: Error) => void
      }
    >()
    vi.mocked(inquiryAIAddInfo).mockImplementation(
      (_images, tags) =>
        new Promise((resolve, reject) => {
          pending.set(tags![0], { resolve, reject })
        })
    )
    const { state, element } = mount<{
      imageUrls: string[]
      inquiryAI: () => Promise<void>
    }>(Add)
    state.imageUrls.push('data:image/png;base64,fixture')
    const recognition = state.inquiryAI()
    await nextTick()
    const group = (label: string) =>
      [...element.querySelectorAll('.form-group')].find(
        (node) => node.querySelector('label')?.textContent === label
      )!
    expect(group('题型').querySelectorAll('.q-spinner')).toHaveLength(1)
    expect(group('题目').querySelectorAll('.q-spinner')).toHaveLength(1)
    pending.get(first)!.resolve([])
    await vi.waitFor(() =>
      expect(
        group(first === 'question_type' ? '题型' : '题目').querySelector(
          '.q-spinner'
        )
      ).toBeNull()
    )
    expect(
      group(first === 'question_type' ? '题目' : '题型').querySelectorAll(
        '.q-spinner'
      )
    ).toHaveLength(1)
    for (const [tag, promise] of pending) {
      if (tag !== first) promise.reject(new Error('Recognition failed'))
    }
    await recognition
    await nextTick()
    expect(element.querySelector('.form-group .q-spinner')).toBeNull()
  }
)

it('ignores AI results from before a form reset', async () => {
  vi.spyOn(llm, 'isConfigured').mockReturnValue(true)
  let resolve!: (results: TaggedResult[]) => void
  vi.mocked(inquiryAIAddInfo).mockReturnValue(
    new Promise((done) => {
      resolve = done
    })
  )
  const { state } = mount<{
    form: { prompt: string; answer: string }
    imageUrls: string[]
    inquiryAI: () => Promise<void>
    resetForm: () => void
  }>(Add)
  state.imageUrls.push('data:image/png;base64,fixture')
  const recognition = state.inquiryAI()
  state.resetForm()
  resolve([
    {
      tag: 'question_text',
      content: 'Stale',
      success: true,
      parsedContent: 'Stale'
    }
  ])
  await recognition
  expect(state.form.prompt).toBe('')
  expect(state.form.answer).toBe('')
})

it('edits and clears nullable fields and relationships, then deletes through the Current API', async () => {
  const { element, state } = mount<{
    editForm: {
      analysis: string
      note: string
      subjectId: string
      sourceId: string
    }
    sourceSelection: { kind: string }
    tempErrorTags: unknown[]
  }>(Detail)
  await vi.waitFor(() => expect(element.textContent).toContain('PAGE_STEM'))
  element.querySelector<HTMLButtonElement>('button.edit-btn')!.click()
  await nextTick()
  Object.assign(state.editForm, {
    analysis: '',
    note: '',
    subjectId: '',
    sourceId: ''
  })
  state.sourceSelection = { kind: 'none' }
  state.tempErrorTags = []
  element.querySelector<HTMLButtonElement>('button.save-btn')!.click()
  await vi.waitFor(() => expect(savedQuestions[0].sourceId).toBeNull())
  expect(savedQuestions[0]).toMatchObject({
    explanation: null,
    note: null,
    tagIds: [],
    attachmentIds: []
  })
  await vi.waitFor(() =>
    expect(element.querySelector('button.save-btn')).toBeNull()
  )
  element.querySelector<HTMLButtonElement>('button.delete-btn')!.click()
  await nextTick()
  document.body.querySelector<HTMLButtonElement>('button.btn-confirm')!.click()
  await vi.waitFor(() => expect(savedQuestions).toHaveLength(0))
})

it('loads statistics using Current library and SRS responses', async () => {
  const { state } = mount<{
    loading: boolean
    questionTotal: number
    dueCount: number
  }>(Profile)
  await vi.waitFor(() => expect(state.loading).toBe(false))
  expect(state.questionTotal).toBe(1)
  expect(state.dueCount).toBe(1)
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === 'get_library_statistics')
  ).toHaveLength(1)
})

it('keeps heatmap range synchronized through empty and unrelated card data', async () => {
  const { state } = mount<{
    loading: boolean
    allCards: SrsData[]
    heatmapData: { buckets: { count: number }[] }[]
    difficultyRange: { min: number; max: number } | null
  }>(Profile)
  await vi.waitFor(() => expect(state.loading).toBe(false))
  expect(state.difficultyRange).toEqual({ min: 5, max: 5 })
  expect(
    state.heatmapData
      .flatMap((row) => row.buckets)
      .reduce((total, bucket) => total + bucket.count, 0)
  ).toBe(1)
  state.allCards = []
  expect(state.difficultyRange).toBeNull()
  expect(state.heatmapData).toEqual([])
  state.allCards = [{ ...srs, questionId: 'missing' }]
  expect(state.difficultyRange).toBeNull()
  expect(state.heatmapData).toEqual([])
  state.allCards = [{ ...srs, difficulty: 7 }]
  expect(state.difficultyRange).toEqual({ min: 7, max: 7 })
})

it('submits review with an RFC 3339 timestamp and consumes the Current result', async () => {
  setReviewQueue([
    { questionId: question.id, question, srs, subjectName: '数学' }
  ])
  const { state } = mount<{
    submitReview: () => Promise<void>
    lastResult: { nextIntervalDays: number } | null
  }>(Review)
  const pending = state.submitReview()
  await vi.waitFor(() => expect(state.lastResult?.nextIntervalDays).toBe(7))
  const request = vi
    .mocked(invoke)
    .mock.calls.find(([command]) => command === 'submit_review')![1] as {
    request: { reviewedAt: string }
  }
  expect(Number.isNaN(Date.parse(request.request.reviewedAt))).toBe(false)
  await pending
  expect(navigation.replace).toHaveBeenCalledWith({ name: 'Preview' })
})

it('reports batch import failures instead of counting them as successful questions', async () => {
  failCreate = true
  const { state } = mount<{ step: string; reviewSubjectId: string }>(
    ImportModal,
    {
      initialData: {
        version: '1.0',
        questions: [
          { prompt: 'NEW_ONE', answer: '', analysis: '' },
          { prompt: 'NEW_TWO', answer: '', analysis: '' }
        ]
      }
    }
  )
  await vi.waitFor(() => expect(state.step).toBe('review'))
  state.reviewSubjectId = 'subject'
  await nextTick()
  document.body
    .querySelector<HTMLButtonElement>('button.import-all-btn')!
    .click()
  await vi.waitFor(() => expect(state.step).toBe('result'))
  expect(document.body.textContent).toContain('失败 2 题')
  expect(document.body.textContent).not.toContain('成功 2 题')
  expect(savedQuestions).toHaveLength(1)
})
