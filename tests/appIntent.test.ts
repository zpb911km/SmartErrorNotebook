// @vitest-environment happy-dom
import { Quasar } from 'quasar'
import { afterEach, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, watch } from 'vue'

import App from '../src/App.vue'
import { quasarOptions } from '../src/quasar'
import router from '../src/router/index'
import { goQuestionList } from '../src/router/navigation'
import { parseQuestionListQuery } from '../src/router/types'

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(async () => vi.fn()) }))
vi.mock('../src/api/platformExceptions', () => ({
  getOpenedUrls: vi.fn(async () => [])
}))
vi.mock('../src/composables/useTheme', () => ({
  initializeTheme: () => ({ dispose: vi.fn() })
}))
vi.mock('../src/components/AppNavigation.vue', () => ({
  default: { render: () => null },
  routePresentation: {
    home: { title: '首页' },
    'question-list': { title: '错题管理' },
    settings: { title: '设置' }
  }
}))
vi.mock('../src/router/index', async () => {
  const { createMemoryHistory, createRouter } = await import('vue-router')
  const { parseQuestionListQuery } = await import('../src/router/types')
  const appRouter = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'home', component: { render: () => null } },
      {
        path: '/question/list',
        name: 'question-list',
        component: { render: () => null },
        props: (route) => parseQuestionListQuery(route.query)
      },
      {
        path: '/settings',
        name: 'settings',
        component: { render: () => null }
      }
    ]
  })
  return {
    default: appRouter,
    goQuestionList: (
      query: { intent?: 'search' | 'import' } = {},
      options: { replace?: boolean; force?: boolean } = {}
    ) =>
      appRouter.push({
        name: 'question-list',
        query: query.intent ? { [query.intent]: null } : {},
        ...options
      })
  }
})

const cleanup: Array<() => void> = []
afterEach(() => {
  cleanup
    .splice(0)
    .reverse()
    .forEach((fn) => fn())
})
async function flush() {
  for (let i = 0; i < 6; i++) await nextTick()
}

it('maps route intent to props and lets the page clear it without remounting', async () => {
  const received: string[] = []
  let mounts = 0
  const page = defineComponent({
    props: { intent: { type: String, default: undefined } },
    setup(props) {
      mounts++
      watch(
        () => props.intent,
        (intent) => {
          if (intent) {
            received.push(intent)
            void goQuestionList({}, { replace: true })
          }
        },
        { immediate: true, flush: 'post' }
      )
      return () => h('div', { 'data-intent': props.intent ?? '' })
    }
  })
  router.addRoute({
    path: '/question/list',
    name: 'question-list',
    component: page,
    props: (route) => parseQuestionListQuery(route.query)
  })
  await router.push('/')
  const element = document.createElement('div')
  document.body.append(element)
  const app = createApp(App).use(router).use(Quasar, quasarOptions)
  app.mount(element)
  cleanup.push(() => {
    app.unmount()
    element.remove()
  })
  for (const intent of ['search', 'search', 'import', 'import'] as const) {
    await goQuestionList({ intent }, { force: true, replace: true })
    await flush()
    await vi.waitFor(() => expect(router.currentRoute.value.query).toEqual({}))
    expect(
      element.querySelector('[data-intent]')?.getAttribute('data-intent')
    ).toBe('')
  }
  expect(received).toEqual(['search', 'search', 'import', 'import'])
  expect(mounts).toBe(1)
  await router.push({
    name: 'question-list',
    query: { search: null, import: null }
  })
  await flush()
  expect(received).toHaveLength(4)
})
