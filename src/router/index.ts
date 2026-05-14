import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/home'
  },
  {
    path: '/home',
    name: 'Home',
    component: () => import('../views/Home.vue'),
    meta: { title: '首页' }
  },
  {
    path: '/add',
    name: 'Add',
    component: () => import('../views/Add.vue'),
    meta: { title: '添加错题' }
  },
  {
    path: '/manage',
    name: 'Manage',
    component: () => import('../views/Manage.vue'),
    meta: { title: '错题管理' }
  },
  {
    path: '/manage-detail/:id',
    name: 'ManageDetail',
    component: () => import('../views/Manage-Detail.vue'),
    meta: { title: '错题详情管理' }
  },
  {
    path: '/review',
    name: 'Preview',
    component: () => import('../views/Preview.vue'),
    meta: { title: '复习计划' }
  },
  {
    path: '/review-detail',
    name: 'ReviewDetail',
    component: () => import('../views/Review-Detail.vue'),
    meta: { title: '复习详情' }
  },
  {
    path: '/stats',
    name: 'Profile',
    component: () => import('../views/Profile.vue'),
    meta: { title: '个人主页' }
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/Settings.vue'),
    meta: { title: '设置' }
  },
  {
    path: '/markdown-test',
    name: 'MarkdownTextareaTest',
    component: () => import('../views/MarkdownTextareaTest.vue'),
    meta: { title: 'Markdown 组件测试' }
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router