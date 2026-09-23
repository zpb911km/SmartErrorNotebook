// @vitest-environment happy-dom
import { invoke } from '@tauri-apps/api/core'
import { Quasar } from 'quasar'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import {
  type Component,
  createApp,
  defineComponent,
  h,
  nextTick,
  reactive,
  ref
} from 'vue'

import AppIcon from '../src/components/AppIcon.vue'
import ImportModal from '../src/components/ImportModal.vue'
import { quasarOptions } from '../src/quasar'
import { llm } from '../src/services/llm'
import { clearReviewQueue, setReviewQueue } from '../src/services/reviewStore'
import { importStore } from '../src/stores/importStore'
import type { Question, SrsData } from '../src/types'
import { inquiryAIAddInfo, type TaggedResult } from '../src/utils/inquiry'
import Home from '../src/views/HomeView.vue'
import Profile from '../src/views/ProfileView.vue'
import Add from '../src/views/QuestionCreateView.vue'
import Detail from '../src/views/QuestionDetailView.vue'
import Manage from '../src/views/QuestionListView.vue'
import Preview from '../src/views/ReviewPlanView.vue'
import Review from '../src/views/ReviewSessionView.vue'

vi.mock('../src/utils/inquiry', () => ({ inquiryAIAddInfo: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('../src/utils/dialog', () => ({
  confirmAction: vi.fn(async () => true),
  showAlert: vi.fn()
}))
vi.mock('../src/components/AppIcon.vue', () => ({
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
const routeState = vi.hoisted(() => ({
  current: {} as Record<string, unknown>
}))
const navigation = vi.hoisted(() => ({
  push: vi.fn(),
  replace: vi.fn(),
  back: vi.fn()
}))
vi.mock('../src/router/index', () => ({
  default: {
    ...navigation,
    currentRoute: {
      get value() {
        return routeState.current
      }
    }
  },
  goBack: () => navigation.back(),
  goQuestionList: (
    query: { intent?: 'search' | 'import' } = {},
    options: { replace?: boolean; force?: boolean } = {}
  ) =>
    navigation.push({
      name: 'question-list',
      query: query.intent ? { [query.intent]: null } : {},
      ...options
    }),
  goQuestionDetail: (id: string) =>
    navigation.push({ name: 'question-detail', params: { id } }),
  goReviewPlan: (options: { replace?: boolean; force?: boolean } = {}) =>
    navigation.push({ name: 'review-plan', ...options }),
  goReviewSession: () => navigation.push({ name: 'review-session' })
}))
vi.mock('vue-router', () => ({
  useRouter: () => ({ ...navigation, currentRoute: { value: currentRoute } }),
  useRoute: () => currentRoute
}))
const currentRoute = reactive({
  name: 'question-list',
  params: { id: 'question' },
  query: {} as Record<string, string | null | undefined>,
  path: '/question/list',
  hash: '',
  meta: {}
})

routeState.current = currentRoute

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
  const input = reactive({
    ...(component === Detail ? { id: 'question' } : {}),
    ...props
  })
  const instance = ref()
  const app = createApp(() => h(component, { ...input, ref: instance }))
  app.use(Quasar, quasarOptions)
  app.component('AppIcon', AppIcon)
  app.component(
    'MarkdownTextarea',
    defineComponent({
      props: { modelValue: { type: String, default: '' } },
      setup: (props) => () => props.modelValue
    })
  )
  app.directive('ripple', {})
  app.directive('scroll-reveal', {})
  app.config.errorHandler = (error) => failures.push(error)
  app.mount(element)
  dispose.push(() => {
    app.unmount()
    element.remove()
  })
  return {
    element,
    state: instance.value.$.setupState as T,
    setProps: (values: Record<string, unknown>) => Object.assign(input, values)
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  currentRoute.name = 'question-list'
  currentRoute.params.id = 'question'
  currentRoute.query = {}
  currentRoute.hash = ''
  navigation.replace.mockImplementation(
    async (target: {
      query?: Record<string, string | null | undefined>
      hash?: string
    }) => {
      if (target.query) currentRoute.query = { ...target.query }
      currentRoute.hash = target.hash ?? ''
    }
  )
  importStore.pendingData = null
  navigation.push.mockImplementation(
    async (target: {
      query?: Record<string, string | null | undefined>
      hash?: string
    }) => {
      if (target.query) currentRoute.query = { ...target.query }
      currentRoute.hash = target.hash ?? ''
    }
  )
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
        return {
          tags: [{ id: 'tag', name: '[已删除]计算', color: '#123456' }]
        }
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
  importStore.pendingData = null
  expect(failures).toEqual([])
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

it('renders management and review lists from Current response fields without write commands', async () => {
  const manage = mount<{
    libraryData: { tags: Array<{ id: string; name: string }> }
  }>(Manage)
  const preview = mount(Preview)
  await vi.waitFor(() => {
    expect(manage.element.textContent).toContain('PAGE_STEM')
    expect(preview.element.textContent).toContain('PAGE_STEM')
    expect(manage.element.textContent).toContain('数学')
    expect(manage.element.textContent).toContain('[已删除]计算')
  })
  expect(manage.state.libraryData.tags).toContainEqual(
    expect.objectContaining({ id: 'tag', name: '[已删除]计算' })
  )
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(([command]) => command.startsWith('list_'))
  ).toBe(true)
})

it('keeps only the latest home statistics and derives overview items', async () => {
  const invokeImplementation = vi.mocked(invoke).getMockImplementation()!
  let resolveFirst!: (value: {
    statistics: {
      questionTotal: number
      cardTotal: number
      dueCount: number
      newCardCount: number
      averageStability: number
      averageDifficulty: number
      totalReviews: number
    }
  }) => void
  const firstResponse = new Promise<{
    statistics: {
      questionTotal: number
      cardTotal: number
      dueCount: number
      newCardCount: number
      averageStability: number
      averageDifficulty: number
      totalReviews: number
    }
  }>((resolve) => {
    resolveFirst = resolve
  })
  let statisticsCalls = 0
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command !== 'get_library_statistics') {
      return invokeImplementation(command, args)
    }
    statisticsCalls++
    if (statisticsCalls === 1) return firstResponse
    return {
      statistics: {
        questionTotal: 2,
        cardTotal: 2,
        dueCount: 3,
        newCardCount: 0,
        averageStability: 4,
        averageDifficulty: 5,
        totalReviews: 4
      }
    }
  })

  const home = mount<{
    load: () => Promise<void | undefined>
    statistics: { questionTotal: number } | null
    overviewItems: Array<{ label: string; value?: number; icon: string }>
    isLoading: boolean
  }>(Home)
  await vi.waitFor(() => expect(statisticsCalls).toBe(1))
  await home.state.load()
  expect(home.state.statistics?.questionTotal).toBe(2)
  expect(home.state.overviewItems.map((item) => item.value)).toEqual([2, 3, 4])

  resolveFirst({
    statistics: {
      questionTotal: 9,
      cardTotal: 9,
      dueCount: 9,
      newCardCount: 9,
      averageStability: 9,
      averageDifficulty: 9,
      totalReviews: 9
    }
  })
  await nextTick()
  await nextTick()
  expect(home.state.statistics?.questionTotal).toBe(2)
  expect(home.state.isLoading).toBe(false)
})

it('recovers home statistics after a failed load', async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error('statistics failed'))
  const home = mount<{
    load: () => Promise<void | undefined>
    statistics: { questionTotal: number } | null
    isLoading: boolean
    error: string
  }>(Home)
  await vi.waitFor(() =>
    expect(home.state.error).toBe('暂时无法读取学习数据，请重试。')
  )
  expect(home.state.isLoading).toBe(false)
  expect(home.state.statistics).toBeNull()

  await home.state.load()
  expect(home.state.error).toBe('')
  expect(home.state.isLoading).toBe(false)
  expect(home.state.statistics?.questionTotal).toBe(1)
})

it('keeps question-list loading data unified across failure and retry', async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error('list failed'))
  const manage = mount<{
    libraryData: {
      questions: unknown[]
      subjects: unknown[]
      sources: unknown[]
      tags: unknown[]
      status: string
    }
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('题目加载失败，请重试。')
  )
  expect(manage.state.libraryData).toMatchObject({
    questions: [],
    status: 'error'
  })
  await vi.waitFor(() =>
    expect(manage.state.libraryData.subjects).toHaveLength(1)
  )

  manage.element
    .querySelector<HTMLButtonElement>('[role="alert"] button')!
    .click()
  await vi.waitFor(() => {
    expect(manage.element.textContent).toContain('PAGE_STEM')
    expect(manage.state.libraryData.status).toBe('ready')
  })
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === 'list_sources')
  ).toHaveLength(1)
})

it('keeps only the latest question-list load result', async () => {
  const invokeImplementation = vi.mocked(invoke).getMockImplementation()!
  let resolveFirst!: (value: { items: Question[]; total: number }) => void
  const firstPage = new Promise<{ items: Question[]; total: number }>(
    (resolve) => {
      resolveFirst = resolve
    }
  )
  let listCalls = 0
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command !== 'list_questions') return invokeImplementation(command, args)
    listCalls++
    if (listCalls === 1) return firstPage
    return { items: [{ ...question, stem: 'LATEST_STEM' }], total: 1 }
  })

  const manage = mount<{
    filterPanelModel: { filter: { keyword: string } }
    libraryData: { status: string }
  }>(Manage)
  await vi.waitFor(() => expect(listCalls).toBe(1))
  manage.state.filterPanelModel.filter.keyword = 'LATEST'
  await vi.waitFor(() => expect(listCalls).toBe(2))
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('LATEST_STEM')
  )

  resolveFirst({ items: [{ ...question, stem: 'STALE_STEM' }], total: 1 })
  await nextTick()
  await nextTick()
  expect(manage.element.textContent).toContain('LATEST_STEM')
  expect(manage.element.textContent).not.toContain('STALE_STEM')
  expect(manage.state.libraryData).toMatchObject({
    status: 'ready'
  })
})

it('does not commit a question-list response after unmount', async () => {
  const invokeImplementation = vi.mocked(invoke).getMockImplementation()!
  let resolvePage!: (value: { items: Question[]; total: number }) => void
  const page = new Promise<{ items: Question[]; total: number }>((resolve) => {
    resolvePage = resolve
  })
  vi.mocked(invoke).mockImplementation((command, args) =>
    command === 'list_questions' ? page : invokeImplementation(command, args)
  )

  const manage = mount<{
    libraryData: { status: string; questions: Question[] }
  }>(Manage)
  await vi.waitFor(() =>
    expect(
      vi
        .mocked(invoke)
        .mock.calls.some(([command]) => command === 'list_questions')
    ).toBe(true)
  )
  dispose.pop()!()
  resolvePage({ items: [question], total: 1 })
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(manage.state.libraryData.status).toBe('loading')
  expect(manage.state.libraryData.questions).toEqual([])
})

it('does not expose stale library cards or export actions during a filter load', async () => {
  const invokeImplementation = vi.mocked(invoke).getMockImplementation()!
  let resolveFiltered!: (value: { items: Question[]; total: number }) => void
  const filteredPage = new Promise<{ items: Question[]; total: number }>(
    (resolve) => {
      resolveFiltered = resolve
    }
  )
  let listCalls = 0
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command !== 'list_questions') return invokeImplementation(command, args)
    listCalls++
    if (listCalls === 2) return filteredPage
    return invokeImplementation(command, args)
  })

  const manage = mount<{
    filterPanelModel: { filter: { keyword: string } }
    libraryData: { status: string }
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('PAGE_STEM')
  )
  manage.state.filterPanelModel.filter.keyword = 'NEW'
  await nextTick()
  expect(listCalls).toBe(1)
  expect(manage.state.libraryData.status).toBe('loading')
  expect(manage.element.querySelector('.action-bar')).toBeNull()
  await vi.waitFor(() => expect(listCalls).toBe(2))
  expect(manage.state.libraryData.status).toBe('loading')
  expect(manage.element.querySelector('.error-list')).toBeNull()
  expect(manage.element.querySelector('.action-bar')).toBeNull()
  expect(manage.element.textContent).not.toContain('PAGE_STEM')

  resolveFiltered({ items: [{ ...question, stem: 'NEW_STEM' }], total: 1 })
  await vi.waitFor(() => expect(manage.state.libraryData.status).toBe('ready'))
  expect(vi.mocked(console.error).mock.calls).toEqual([])
  await vi.waitFor(() => {
    expect(manage.element.textContent).toContain('NEW_STEM')
    expect(manage.element.querySelector('.action-bar')).not.toBeNull()
  })
})

it('debounces keyword edits and reuses list resources across searches', async () => {
  const manage = mount<{
    filterPanelModel: { filter: { keyword: string } }
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('PAGE_STEM')
  )

  manage.state.filterPanelModel.filter.keyword = 'P'
  await nextTick()
  manage.state.filterPanelModel.filter.keyword = 'PA'
  await nextTick()
  manage.state.filterPanelModel.filter.keyword = 'PAGE'
  await nextTick()

  const calls = vi.mocked(invoke).mock.calls
  expect(
    calls.filter(([command]) => command === 'list_questions')
  ).toHaveLength(1)
  await vi.waitFor(() =>
    expect(
      calls.filter(([command]) => command === 'list_questions')
    ).toHaveLength(2)
  )
  expect(
    calls.filter(([command]) => command === 'list_questions').at(-1)
  ).toEqual([
    'list_questions',
    expect.objectContaining({
      request: expect.objectContaining({ filter: { keyword: 'PAGE' } })
    })
  ])
  for (const command of ['list_sources', 'list_subjects', 'list_tags']) {
    expect(calls.filter(([name]) => name === command)).toHaveLength(1)
  }
  expect(calls.filter(([name]) => name === 'list_srs_data')).toHaveLength(2)
})

it('refreshes SRS data for each completed list query', async () => {
  const manage = mount<{
    filterPanelModel: { filter: { keyword: string } }
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.querySelector('.error-list')?.textContent).toContain(
      '中等'
    )
  )

  reviewCards = [{ ...srs, difficulty: 8 }]
  manage.state.filterPanelModel.filter.keyword = 'PAGE'
  await vi.waitFor(() =>
    expect(manage.element.querySelector('.error-list')?.textContent).toContain(
      '困难'
    )
  )
  const calls = vi.mocked(invoke).mock.calls
  expect(calls.filter(([name]) => name === 'list_srs_data')).toHaveLength(2)
  expect(calls.filter(([name]) => name === 'list_sources')).toHaveLength(1)
})

it('runs an immediate filter query and cancels a pending keyword query', async () => {
  const manage = mount<{
    filterPanelModel: { filter: { keyword: string; book: string } }
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('PAGE_STEM')
  )

  manage.state.filterPanelModel.filter.keyword = 'PAGE'
  await nextTick()
  manage.state.filterPanelModel.filter.book = '教材'
  await vi.waitFor(() => {
    const listCalls = vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === 'list_questions')
    expect(listCalls).toHaveLength(2)
    expect(listCalls[1][1]).toEqual({
      request: expect.objectContaining({
        filter: expect.objectContaining({ keyword: 'PAGE', book: '教材' })
      })
    })
  })
  await new Promise((resolve) => setTimeout(resolve, 300))
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === 'list_questions')
  ).toHaveLength(2)
})

it('retries resource requests after a failed library load', async () => {
  const invokeImplementation = vi.mocked(invoke).getMockImplementation()!
  let sourceCalls = 0
  vi.mocked(invoke).mockImplementation((command, args) => {
    if (command === 'list_sources' && ++sourceCalls === 1) {
      return Promise.reject(new Error('sources unavailable'))
    }
    return invokeImplementation(command, args)
  })

  const manage = mount(Manage)
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('题目加载失败，请重试。')
  )
  manage.element
    .querySelector<HTMLButtonElement>('[role="alert"] button')!
    .click()
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('PAGE_STEM')
  )
  expect(sourceCalls).toBe(2)
})

it('sends supported library filters to Current IPC and keeps subject and difficulty local', async () => {
  const manage = mount<{
    filterPanelModel: {
      filter: {
        subjectId: string
        book: string
        chapter: string
        knowledge: string
        keyword?: string
        tagIds?: string[]
        dateRange?: string
      }
      sort: { difficulty?: string; mastery?: string }
    }
  }>(Manage)
  await vi.waitFor(() =>
    expect(
      vi
        .mocked(invoke)
        .mock.calls.some(([command]) => command === 'list_questions')
    ).toBe(true)
  )

  Object.assign(manage.state.filterPanelModel.filter, {
    subjectId: 'subject',
    book: '教材',
    chapter: '第一章',
    knowledge: '函数',
    keyword: ' 极限 ',
    tagIds: ['tag'],
    dateRange: '7days'
  })
  manage.state.filterPanelModel.sort.mastery = 'asc'

  await vi.waitFor(() => {
    const calls = vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === 'list_questions')
    expect(calls.length).toBeGreaterThan(1)
    const request = calls.at(-1)?.[1] as {
      request: {
        filter: Record<string, unknown>
        sort: string[]
      }
    }
    expect(request.request.filter).toMatchObject({
      keyword: ' 极限 ',
      book: '教材',
      chapter: '第一章',
      knowledge: '函数',
      tagIds: ['tag']
    })
    expect(request.request.filter.updatedSince).toEqual(expect.any(String))
    expect(request.request.filter).not.toHaveProperty('subjectId')
    expect(request.request.sort).toEqual([
      'MASTERY_ASC',
      'UPDATED_AT_DESC',
      'ID_ASC'
    ])
    expect(request.request.sort).not.toContain('DIFFICULTY_DESC')
  })
})

it('sorts difficulty locally without requesting a new question list', async () => {
  savedQuestions = [
    { ...question, id: 'hard', stem: 'HARD_STEM' },
    { ...question, id: 'easy', stem: 'EASY_STEM' }
  ]
  reviewCards = [
    { ...srs, questionId: 'hard', difficulty: 8 },
    { ...srs, questionId: 'easy', difficulty: 2 }
  ]
  const manage = mount<{
    filterPanelModel: { sort: { difficulty: string } }
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.querySelectorAll('.error-card')).toHaveLength(2)
  )
  const listCalls = () =>
    vi.mocked(invoke).mock.calls.filter(([name]) => name === 'list_questions')
  expect(listCalls()).toHaveLength(1)

  manage.state.filterPanelModel.sort.difficulty = 'asc'
  await nextTick()
  expect(manage.element.querySelector('.error-card')?.textContent).toContain(
    'EASY_STEM'
  )
  expect(listCalls()).toHaveLength(1)

  manage.state.filterPanelModel.sort.difficulty = 'desc'
  await nextTick()
  expect(manage.element.querySelector('.error-card')?.textContent).toContain(
    'HARD_STEM'
  )
  expect(listCalls()).toHaveLength(1)
})

it('uses only the question ID for navigation and batch selection', async () => {
  const manage = mount<{
    onQuestionCardClick: (id: string) => void
    selectAllQuestions: () => void
    isSelectionMode: boolean
    selectedQuestionIds: Set<string>
  }>(Manage)
  await vi.waitFor(() =>
    expect(manage.element.textContent).toContain('PAGE_STEM')
  )

  manage.state.onQuestionCardClick('question')
  expect(navigation.push).toHaveBeenLastCalledWith({
    name: 'question-detail',
    params: { id: 'question' }
  })

  navigation.push.mockClear()
  manage.state.isSelectionMode = true
  await nextTick()
  const checkbox =
    manage.element.querySelector<HTMLElement>('[role="checkbox"]')!
  const exportButton = Array.from(
    manage.element.querySelectorAll<HTMLButtonElement>('button')
  ).find((button) => button.textContent?.includes('导出'))!
  expect(exportButton.disabled).toBe(true)
  checkbox.click()
  await nextTick()
  expect(manage.state.selectedQuestionIds).toBeInstanceOf(Set)
  expect(manage.state.selectedQuestionIds.has('question')).toBe(true)
  expect(manage.element.textContent).toContain('已选 1 题')
  expect(exportButton.disabled).toBe(false)
  checkbox.click()
  await nextTick()
  expect(manage.state.selectedQuestionIds.has('question')).toBe(false)
  expect(exportButton.disabled).toBe(true)

  manage.state.onQuestionCardClick('question')
  expect(manage.state.selectedQuestionIds.has('question')).toBe(true)
  manage.state.onQuestionCardClick('question')
  expect(manage.state.selectedQuestionIds.has('question')).toBe(false)
  manage.state.selectAllQuestions()
  expect(manage.state.selectedQuestionIds).toEqual(new Set(['question']))
  expect(navigation.push).not.toHaveBeenCalled()
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
  await vi.waitFor(() =>
    expect(navigation.push).toHaveBeenCalledWith({
      name: 'question-list',
      query: {}
    })
  )
})

it('uses history when returning from question details', async () => {
  const { state } = mount<{ goBack: () => void }>(Detail)
  state.goBack()
  expect(navigation.back).toHaveBeenCalledOnce()
})

it('reloads question details when its own route ID changes', async () => {
  currentRoute.name = 'question-detail'
  const { setProps } = mount(Detail)
  await vi.waitFor(() =>
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('get_question', {
      request: { id: 'question' }
    })
  )
  setProps({ id: 'next-question' })
  await vi.waitFor(() =>
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('get_question', {
      request: { id: 'next-question' }
    })
  )
})

it('handles initial and consecutive imports, then clears the route intent', async () => {
  currentRoute.query = { import: null, keep: 'yes' }
  importStore.pendingData = { questions: [{ prompt: 'Imported' }] }
  const { state, setProps } = mount<{
    activeModal: 'import' | 'export' | null
  }>(Manage, {
    intent: 'import'
  })
  await vi.waitFor(() => expect(state.activeModal).toBe('import'))
  await vi.waitFor(() => expect(currentRoute.query).toEqual({}))
  for (const prompt of ['Second', 'Third']) {
    state.activeModal = null
    await nextTick()
    importStore.pendingData = { questions: [{ prompt }] }
    setProps({ intent: undefined })
    await nextTick()
    setProps({ intent: 'import' })
    await vi.waitFor(() => expect(state.activeModal).toBe('import'))
    await vi.waitFor(() => expect(currentRoute.query).toEqual({}))
  }
  expect(navigation.push).toHaveBeenCalledTimes(3)
  expect(navigation.push).toHaveBeenLastCalledWith({
    name: 'question-list',
    query: {},
    replace: true
  })
})

it('keeps import and export modals mutually exclusive', async () => {
  const { element, state } = mount<{
    activeModal: 'import' | 'export' | null
    onImportModalComplete: () => void
    onImportModalDismiss: () => void
  }>(Manage)
  await vi.waitFor(() => expect(element.textContent).toContain('PAGE_STEM'))

  element.querySelector<HTMLButtonElement>('.action-bar .import-btn')!.click()
  await nextTick()
  expect(state.activeModal).toBe('import')
  expect(document.body.querySelector('.import-modal')).not.toBeNull()
  expect(document.body.querySelector('.export-modal')).toBeNull()

  element.querySelector<HTMLButtonElement>('.action-bar .export-btn')!.click()
  await vi.waitFor(() => {
    expect(state.activeModal).toBe('export')
    expect(document.body.querySelector('.import-modal')).toBeNull()
    expect(
      document.body.querySelectorAll('.import-modal, .export-modal').length
    ).toBeLessThanOrEqual(1)
  })

  state.activeModal = 'import'
  importStore.pendingData = { questions: [{ prompt: 'Imported' }] }
  const listCallsBeforeComplete = vi
    .mocked(invoke)
    .mock.calls.filter(([command]) => command === 'list_questions').length
  const resourceCommands = [
    'list_sources',
    'list_subjects',
    'list_tags',
    'list_srs_data'
  ]
  const resourcesBeforeComplete = resourceCommands.map(
    (command) =>
      vi.mocked(invoke).mock.calls.filter(([name]) => name === command).length
  )
  state.onImportModalComplete()
  expect(state.activeModal).toBe('import')
  await vi.waitFor(() =>
    expect(document.body.querySelector('.import-modal')).not.toBeNull()
  )
  resourceCommands.forEach((command, index) => {
    expect(
      vi.mocked(invoke).mock.calls.filter(([name]) => name === command)
    ).toHaveLength(resourcesBeforeComplete[index] + 1)
  })
  expect(importStore.pendingData).toBeNull()
  await vi.waitFor(() =>
    expect(
      vi
        .mocked(invoke)
        .mock.calls.filter(([command]) => command === 'list_questions').length
    ).toBeGreaterThan(listCallsBeforeComplete)
  )

  importStore.pendingData = { questions: [{ prompt: 'Dismissed' }] }
  state.onImportModalDismiss()
  expect(state.activeModal).toBeNull()
  expect(importStore.pendingData).toBeNull()
})

it('keeps import results visible until the user closes the modal', async () => {
  importStore.pendingData = {
    version: '1.0',
    questions: [{ prompt: 'NEW_ONE', answer: '', analysis: '' }]
  }
  const { element, state } = mount<{
    activeModal: 'import' | 'export' | null
  }>(Manage)
  await vi.waitFor(() => expect(element.textContent).toContain('PAGE_STEM'))
  element.querySelector<HTMLButtonElement>('.action-bar .import-btn')!.click()
  await vi.waitFor(() =>
    expect(document.body.querySelector('.import-modal')?.textContent).toContain(
      'NEW_ONE'
    )
  )
  const modal = document.body.querySelector('.import-modal')!

  modal.querySelector<HTMLButtonElement>('button.skip-btn')!.click()
  await vi.waitFor(() => expect(modal.textContent).toContain('导入完成'))
  expect(state.activeModal).toBe('import')
  expect(document.body.querySelector('.import-modal')).toBe(modal)
  expect(importStore.pendingData).toBeNull()

  modal.querySelector<HTMLButtonElement>('.cancel-btn')!.click()
  await vi.waitFor(() => expect(state.activeModal).toBeNull())
  expect(document.body.querySelector('.import-modal')).toBeNull()
})

it('focuses search on arrival and repeated requests, and stops after unmount', async () => {
  currentRoute.query = { search: null }
  const { element, setProps } = mount(Manage, { intent: 'search' })
  const search = element.querySelector<HTMLInputElement>(
    'input[placeholder^="搜索题干"]'
  )!
  await vi.waitFor(() => expect(document.activeElement).toBe(search))
  for (let count = 0; count < 2; count++) {
    search.blur()
    setProps({ intent: undefined })
    await nextTick()
    setProps({ intent: 'search' })
    await vi.waitFor(() => expect(document.activeElement).toBe(search))
  }
  expect(navigation.push).toHaveBeenCalledTimes(3)
  expect(navigation.push).toHaveBeenLastCalledWith({
    name: 'question-list',
    query: {},
    replace: true
  })
  const focus = vi.spyOn(search, 'focus')
  dispose.pop()!()
  setProps({ intent: 'search' })
  await nextTick()
  expect(focus).not.toHaveBeenCalled()
})

it('replaces an empty review and an explicitly exited review with the review list', async () => {
  mount(Review)
  expect(navigation.push).toHaveBeenCalledWith({
    name: 'review-plan',
    replace: true
  })
  navigation.replace.mockClear()
  setReviewQueue([
    { questionId: question.id, question, srs, subjectName: '数学' }
  ])
  const { state } = mount<{ exitReview: () => void }>(Review)
  state.exitReview()
  expect(navigation.push).toHaveBeenCalledWith({
    name: 'review-plan',
    replace: true
  })
  expect(navigation.replace).not.toHaveBeenCalled()
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
  expect(navigation.push).toHaveBeenCalledWith({
    name: 'review-plan',
    replace: true
  })
})

it('reports batch import failures instead of counting them as successful questions', async () => {
  failCreate = true
  const onComplete = vi.fn()
  const onDismiss = vi.fn()
  const { state } = mount<{ step: string; reviewSubjectId: string }>(
    ImportModal,
    {
      onComplete,
      onDismiss,
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
  expect(onComplete).toHaveBeenCalledOnce()
  expect(document.body.textContent).toContain('失败 2 题')
  expect(document.body.textContent).not.toContain('成功 2 题')
  expect(savedQuestions).toHaveLength(1)
  document.body
    .querySelector<HTMLButtonElement>('.import-modal .cancel-btn')!
    .click()
  expect(onDismiss).toHaveBeenCalledOnce()
})

it('ignores conflicting signals without triggering either action', async () => {
  const { element, state } = mount<{
    activeModal: 'import' | 'export' | null
  }>(Manage)
  await nextTick()
  importStore.pendingData = { questions: [{ prompt: 'Imported' }] }
  currentRoute.hash = '#anchor'
  currentRoute.query = { search: null, import: null, keep: 'yes' }
  await nextTick()
  expect(document.activeElement).not.toBe(
    element.querySelector('input[placeholder^="搜索题干"]')
  )
  expect(state.activeModal).toBeNull()
  expect(importStore.pendingData).not.toBeNull()
  expect(navigation.replace).not.toHaveBeenCalled()
  expect(navigation.push).not.toHaveBeenCalled()
  expect(currentRoute.query).toEqual({
    search: null,
    import: null,
    keep: 'yes'
  })
  expect(currentRoute.hash).toBe('#anchor')
})

it('does not open pending imports without an import prop', async () => {
  currentRoute.query = { search: null, import: null }
  importStore.pendingData = { questions: [{ prompt: 'Imported' }] }
  const { state } = mount<{
    activeModal: 'import' | 'export' | null
  }>(Manage)
  await nextTick()
  expect(currentRoute.query).toEqual({ search: null, import: null })
  expect(state.activeModal).toBeNull()
})

it('does not consume valued or legacy query signals', async () => {
  currentRoute.query = { focus: 'search', search: '' }
  const { element } = mount(Manage)
  await nextTick()
  expect(navigation.replace).not.toHaveBeenCalled()
  expect(document.activeElement).not.toBe(
    element.querySelector('input[placeholder^="搜索题干"]')
  )
  expect(currentRoute.query).toEqual({
    focus: 'search',
    search: ''
  })
})
