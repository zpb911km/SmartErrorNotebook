import type { RouteRecordInfo, RouteRecordRaw } from 'vue-router'

import { parseQuestionListQuery } from './types'

export interface RouteNamedMap {
  home: RouteRecordInfo<
    'home',
    '/',
    Record<never, never>,
    Record<never, never>,
    never
  >
  'question-list': RouteRecordInfo<
    'question-list',
    '/question/list',
    Record<never, never>,
    Record<never, never>,
    never
  >
  'question-create': RouteRecordInfo<
    'question-create',
    '/question/create',
    Record<never, never>,
    Record<never, never>,
    never
  >
  'question-detail': RouteRecordInfo<
    'question-detail',
    '/question/:id',
    { id: string },
    { id: string },
    never
  >
  'review-plan': RouteRecordInfo<
    'review-plan',
    '/review',
    Record<never, never>,
    Record<never, never>,
    never
  >
  'review-session': RouteRecordInfo<
    'review-session',
    '/review/session',
    Record<never, never>,
    Record<never, never>,
    never
  >
  profile: RouteRecordInfo<
    'profile',
    '/profile',
    Record<never, never>,
    Record<never, never>,
    never
  >
  settings: RouteRecordInfo<
    'settings',
    '/settings',
    Record<never, never>,
    Record<never, never>,
    never
  >
  sync: RouteRecordInfo<
    'sync',
    '/settings/sync',
    Record<never, never>,
    Record<never, never>,
    never
  >
  'markdown-playground': RouteRecordInfo<
    'markdown-playground',
    '/dev/markdown',
    Record<never, never>,
    Record<never, never>,
    never
  >
}

// Runtime matching, components and types belong here. Presentation belongs in
// src/components/AppNavigation.vue. Tests compare canonical full paths with RouteNamedMap.
export const routes: RouteRecordRaw[] = [
  {
    name: 'home',
    path: '/',
    alias: '/home',
    sensitive: true,
    strict: true,
    component: () => import('@/views/HomeView.vue')
  },
  {
    path: '/question',
    sensitive: true,
    strict: true,
    redirect: { name: 'question-list' },
    children: [
      {
        name: 'question-list',
        path: 'list',
        sensitive: true,
        strict: true,
        component: () => import('@/views/QuestionListView.vue'),
        props: (route) => parseQuestionListQuery(route.query)
      },
      {
        name: 'question-create',
        path: 'create',
        sensitive: true,
        strict: true,
        component: () => import('@/views/QuestionCreateView.vue')
      },
      {
        name: 'question-detail',
        path: ':id',
        sensitive: true,
        strict: true,
        component: () => import('@/views/QuestionDetailView.vue'),
        props: true
      }
    ]
  },
  {
    path: '/review',
    sensitive: true,
    strict: true,
    children: [
      {
        name: 'review-plan',
        path: '',
        sensitive: true,
        strict: true,
        component: () => import('@/views/ReviewPlanView.vue')
      },
      {
        name: 'review-session',
        path: 'session',
        sensitive: true,
        strict: true,
        component: () => import('@/views/ReviewSessionView.vue')
      }
    ]
  },
  {
    name: 'profile',
    path: '/profile',
    sensitive: true,
    strict: true,
    component: () => import('@/views/ProfileView.vue')
  },
  {
    path: '/settings',
    sensitive: true,
    strict: true,
    children: [
      {
        name: 'settings',
        path: '',
        sensitive: true,
        strict: true,
        component: () => import('@/views/SettingsView.vue')
      },
      {
        name: 'sync',
        path: 'sync',
        sensitive: true,
        strict: true,
        component: () => import('@/views/SyncView.vue')
      }
    ]
  },
  {
    path: '/dev',
    sensitive: true,
    strict: true,
    children: [
      {
        name: 'markdown-playground',
        path: 'markdown',
        sensitive: true,
        strict: true,
        component: () => import('@/views/MarkdownPlaygroundView.vue')
      }
    ]
  }
]
