import { createRouter, createWebHistory } from 'vue-router'

import { routes } from './catalog'

export default createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 })
})

export * from './navigation'
