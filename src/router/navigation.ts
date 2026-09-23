import router from './index'
import { type QuestionListQuery, serializeQuestionListQuery } from './types'

export interface NavigationOptions {
  replace?: boolean
  force?: boolean
}

// Route state is read-only to pages. All programmatic navigation, including
// history traversal, goes through these operations. Pages never clear signals.
export function goHome(options: NavigationOptions = {}) {
  // Vue Router considers an alias the same route and otherwise suppresses
  // navigation. Explicitly going home should use the canonical root URL.
  if (router.currentRoute.value.matched.some((record) => record.aliasOf)) {
    return router.push({ name: 'home', force: true, ...options })
  }
  return router.push({ name: 'home', ...options })
}

export function goQuestionList(
  query: QuestionListQuery = {},
  options: NavigationOptions = {}
) {
  return router.push({
    name: 'question-list',
    query: serializeQuestionListQuery(query),
    ...options
  })
}

export function goQuestionCreate(options: NavigationOptions = {}) {
  return router.push({ name: 'question-create', ...options })
}

export function goQuestionDetail(id: string, options: NavigationOptions = {}) {
  return router.push({ name: 'question-detail', params: { id }, ...options })
}

// Completion/exit opts into replacement to remove the finished session.
export function goReviewPlan(options: NavigationOptions = {}) {
  return router.push({ ...{ name: 'review-plan' }, ...options })
}

export function goReviewSession(options: NavigationOptions = {}) {
  return router.push({ name: 'review-session', ...options })
}

export function goProfile(options: NavigationOptions = {}) {
  return router.push({ name: 'profile', ...options })
}

export function goSettings(options: NavigationOptions = {}) {
  return router.push({ name: 'settings', ...options })
}

export function goSync(options: NavigationOptions = {}) {
  return router.push({ name: 'sync', ...options })
}

export function goMarkdownPlayground(options: NavigationOptions = {}) {
  return router.push({ name: 'markdown-playground', ...options })
}

export function goBack() {
  return router.back()
}
