// @vitest-environment happy-dom
import { readFileSync } from 'node:fs'

import ts from 'typescript'
import { afterEach, expect, it, vi } from 'vitest'
import type { RouteRecordRaw } from 'vue-router'

import { routes } from '../src/router/catalog'
import router from '../src/router/index'
import * as navigation from '../src/router/navigation'

// Replace the application singleton before loading the standalone operations;
// tests never initialize browser Hash history or application page components.
vi.mock('../src/router/index', async () => {
  const { createMemoryHistory, createRouter } = await import('vue-router')
  const { routes } = await import('../src/router/catalog')
  function stubViews(records: RouteRecordRaw[]): RouteRecordRaw[] {
    return records.map(
      (record) =>
        ({
          ...record,
          ...('component' in record
            ? { component: { render: () => null } }
            : {}),
          ...(record.children ? { children: stubViews(record.children) } : {})
        }) as RouteRecordRaw
    )
  }
  return {
    default: createRouter({
      history: createMemoryHistory(),
      routes: stubViews(routes)
    })
  }
})
afterEach(() => vi.restoreAllMocks())

it('uses canonical home, a non-redirecting alias and a useful question group entry', async () => {
  await router.push('/home')
  expect(router.currentRoute.value.name).toBe('home')
  expect(router.currentRoute.value.path).toBe('/home')
  expect(router.currentRoute.value.redirectedFrom).toBeUndefined()
  await navigation.goHome()
  expect(router.currentRoute.value.path).toBe('/')
  await router.push('/question')
  expect(router.currentRoute.value.path).toBe('/question/list')
  expect(router.resolve('/question/create').name).toBe('question-create')
  expect(router.resolve('/question/list').name).toBe('question-list')
  expect(
    router.resolve('/question/one').matched.map((record) => record.path)
  ).toEqual(['/question', '/question/:id'])
})

it('applies strict and sensitive matching to every record and the home alias', () => {
  function check(records: RouteRecordRaw[]) {
    for (const record of records) {
      expect(record.strict, record.path).toBe(true)
      expect(record.sensitive, record.path).toBe(true)
      expect(record.meta, record.path).toBeUndefined()
      if (record.children) check(record.children)
    }
  }
  check(routes)
  vi.spyOn(console, 'warn').mockImplementation(() => {})
  for (const path of [
    '/Home',
    '/home/',
    '/QUESTION/list',
    '/question/list/',
    '/question/create/',
    '/question/one/',
    '/Review',
    '/review/',
    '/review/session/',
    '/Settings',
    '/settings/',
    '/settings/sync/',
    '/dev/markdown/'
  ]) {
    expect(router.resolve(path).matched, path).toHaveLength(0)
  }
  // Dynamic IDs remain case-sensitive data, not static path segments.
  expect(router.resolve('/question/AbC').params).toEqual({ id: 'AbC' })
})

it('does not retain former entry points', () => {
  vi.spyOn(console, 'warn').mockImplementation(() => {})
  for (const path of [
    '/questions',
    '/questions/new',
    '/questions/one',
    '/add',
    '/manage',
    '/manage-detail/one',
    '/review-detail',
    '/stats',
    '/sync',
    '/community',
    '/markdown-test'
  ]) {
    expect(router.resolve(path).matched, path).toHaveLength(0)
  }
})

it('uses explicit push actions and optional replacement without setup injection', async () => {
  const push = vi.spyOn(router, 'push')
  const replace = vi.spyOn(router, 'replace')
  const actions = [
    [navigation.goHome, 'home'],
    [navigation.goQuestionList, 'question-list'],
    [navigation.goQuestionCreate, 'question-create'],
    [navigation.goReviewPlan, 'review-plan'],
    [navigation.goReviewSession, 'review-session'],
    [navigation.goProfile, 'profile'],
    [navigation.goSettings, 'settings'],
    [navigation.goSync, 'sync'],
    [navigation.goMarkdownPlayground, 'markdown-playground']
  ] as const
  for (const [go, name] of actions) {
    await go()
    expect(push).toHaveBeenLastCalledWith(
      name === 'question-list' ? { name, query: {} } : { name }
    )
  }
  expect(replace).not.toHaveBeenCalled()
  const id = '题目/a ?#%'
  await navigation.goQuestionDetail(id)
  expect(push).toHaveBeenLastCalledWith({
    name: 'question-detail',
    params: { id }
  })
  expect(router.currentRoute.value.path).toBe(
    '/question/' + encodeURIComponent(id)
  )
  expect(router.currentRoute.value.params).toEqual({ id })
  await navigation.goQuestionDetail('second')
  expect(router.currentRoute.value.params).toEqual({ id: 'second' })
  await navigation.goReviewPlan({ replace: true })
  expect(push).toHaveBeenCalledWith({ name: 'review-plan', replace: true })
})

it('keeps a completed review out of history', async () => {
  await navigation.goHome()
  await navigation.goReviewSession()
  await navigation.goReviewPlan({ replace: true })
  router.back()
  await vi.waitFor(() => expect(router.currentRoute.value.name).toBe('home'))
})

it.each(['search', 'import'] as const)(
  'navigates with %s intent without inheriting local query',
  async (intent) => {
    await navigation.goHome()
    const push = vi.spyOn(router, 'push')
    const replace = vi.spyOn(router, 'replace')
    const signal = { [intent]: null }
    await navigation.goQuestionList({ intent })
    expect(push).toHaveBeenLastCalledWith({
      name: 'question-list',
      query: signal
    })
    expect(replace).not.toHaveBeenCalled()
    expect(router.currentRoute.value.fullPath).toBe('/question/list?' + intent)
    await router.replace({ query: { keep: 'yes' }, hash: '#anchor' })
    await navigation.goQuestionList({ intent })
    const target = {
      name: 'question-list',
      query: signal
    }
    expect(push).toHaveBeenLastCalledWith({
      ...target
    })
    expect(router.currentRoute.value.hash).toBe('')
    await navigation.goQuestionList({ intent }, { replace: false })
    expect(push).toHaveBeenLastCalledWith({
      ...target,
      replace: false
    })
    await navigation.goHome()
    await navigation.goQuestionList({ intent }, { replace: true })
    expect(push).toHaveBeenLastCalledWith({
      name: 'question-list',
      query: signal,
      replace: true
    })
  }
)

it('keeps runtime canonical paths aligned with RouteNamedMap, excluding aliases', () => {
  const source = ts.createSourceFile(
    'catalog.ts',
    readFileSync('src/router/catalog.ts', 'utf8'),
    ts.ScriptTarget.Latest,
    true
  )
  const expected: Record<string, string> = {}
  for (const statement of source.statements) {
    if (
      !ts.isInterfaceDeclaration(statement) ||
      statement.name.text !== 'RouteNamedMap'
    )
      continue
    for (const member of statement.members) {
      if (
        !ts.isPropertySignature(member) ||
        !member.type ||
        !ts.isTypeReferenceNode(member.type)
      )
        continue
      const [name, path] = member.type.typeArguments ?? []
      if (
        !name ||
        !path ||
        !ts.isLiteralTypeNode(name) ||
        !ts.isStringLiteral(name.literal) ||
        !ts.isLiteralTypeNode(path) ||
        !ts.isStringLiteral(path.literal)
      )
        throw new Error('Expected literal route name and path')
      expect(member.name.getText(source).replace(/^['"]|['"]$/g, '')).toBe(
        name.literal.text
      )
      expected[name.literal.text] = path.literal.text
    }
  }
  expect(Object.keys(expected)).toHaveLength(10)
  const canonical = router
    .getRoutes()
    .filter((record) => record.name && !record.aliasOf)
  expect(canonical).toHaveLength(10)
  expect(
    Object.fromEntries(canonical.map((record) => [record.name, record.path]))
  ).toEqual(expected)
})
it('allows explicit force false to override home alias normalization', async () => {
  await router.push('/home')
  await navigation.goHome({ force: false })
  expect(router.currentRoute.value.path).toBe('/home')
  await navigation.goHome()
  expect(router.currentRoute.value.path).toBe('/')
})

it('passes shared navigation options through every destination', async () => {
  const push = vi.spyOn(router, 'push')
  const actions = [
    navigation.goHome,
    navigation.goQuestionCreate,
    navigation.goReviewPlan,
    navigation.goReviewSession,
    navigation.goProfile,
    navigation.goSettings,
    navigation.goSync,
    navigation.goMarkdownPlayground,
    (options: navigation.NavigationOptions) =>
      navigation.goQuestionList({}, options),
    (options: navigation.NavigationOptions) =>
      navigation.goQuestionDetail('one', options)
  ]
  for (const go of actions) {
    await go({ replace: true, force: true })
    expect(push).toHaveBeenLastCalledWith(
      expect.objectContaining({ replace: true, force: true })
    )
  }
})
it('replaces the other intent without inheriting unrelated query', async () => {
  await router.push({
    name: 'question-list',
    query: { search: null, keep: 'yes' }
  })
  await navigation.goQuestionList({ intent: 'import' })
  expect(router.currentRoute.value.query).toEqual({ import: null })
  await navigation.goQuestionList({ intent: 'search' })
  expect(router.currentRoute.value.query).toEqual({ search: null })
  await navigation.goQuestionList()
  expect(router.currentRoute.value.query).toEqual({})
})
it.each(['search', 'import'] as const)(
  'makes repeated %s requests observable unless force is disabled',
  async (intent) => {
    await navigation.goQuestionList({ intent })
    const first = router.currentRoute.value.query
    await navigation.goQuestionList({ intent }, { force: true, replace: true })
    expect(router.currentRoute.value.query).not.toBe(first)
    expect(router.currentRoute.value.query).toEqual(first)
    const second = router.currentRoute.value.query
    await navigation.goQuestionList({ intent }, { force: false })
    expect(router.currentRoute.value.query).toBe(second)
  }
)

it('delegates history traversal to the router', () => {
  const back = vi.spyOn(router, 'back').mockImplementation(() => {})
  navigation.goBack()
  expect(back).toHaveBeenCalledOnce()
})
