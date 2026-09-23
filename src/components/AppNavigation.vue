<script lang="ts">
import {
  goHome,
  goProfile,
  goQuestionCreate,
  goQuestionList,
  goReviewPlan,
  goSettings
} from '@/router'
import type { RouteNamedMap } from '@/router/catalog'

type NamedRouteTarget = { name: keyof RouteNamedMap }

type RoutePresentation = {
  title: string
  backTo?: NamedRouteTarget
  navigationItem?: {
    order: number
    label: string
    icon: string
    description: string
    activeOn: ReadonlyArray<keyof RouteNamedMap>
    navigate: () => unknown
  }
}

// UI owns titles, menu presentation and toolbar return behavior. A toolbar
// parent link is deliberately different from a destination's history return.
export const routePresentation: Record<keyof RouteNamedMap, RoutePresentation> =
  {
    home: {
      title: '首页',
      navigationItem: {
        order: 0,
        label: '首页',
        icon: 'space_dashboard',
        description: '学习概览',
        activeOn: ['home'],
        navigate: goHome
      }
    },
    'question-list': {
      title: '错题管理',
      navigationItem: {
        order: 2,
        label: '管理',
        icon: 'inventory_2',
        description: '整理错题与知识',
        activeOn: ['question-list', 'question-detail'],
        navigate: goQuestionList
      }
    },
    'question-create': {
      title: '添加错题',
      navigationItem: {
        order: 1,
        label: '添加',
        icon: 'add_circle_outline',
        description: '记录新的收获',
        activeOn: ['question-create'],
        navigate: goQuestionCreate
      }
    },
    'question-detail': {
      title: '错题详情管理',
      backTo: { name: 'question-list' }
    },
    'review-plan': {
      title: '复习计划',
      navigationItem: {
        order: 3,
        label: '复习',
        icon: 'auto_stories',
        description: '让知识更牢固',
        activeOn: ['review-plan', 'review-session'],
        navigate: goReviewPlan
      }
    },
    'review-session': {
      title: '复习详情',
      backTo: { name: 'review-plan' }
    },
    profile: {
      title: '个人主页',
      navigationItem: {
        order: 4,
        label: '我的',
        icon: 'insights',
        description: '查看学习进展',
        activeOn: ['profile'],
        navigate: goProfile
      }
    },
    settings: { title: '设置', backTo: { name: 'home' } },
    sync: { title: '同步', backTo: { name: 'settings' } },
    'markdown-playground': {
      title: 'Markdown 组件测试',
      backTo: { name: 'home' }
    }
  }
</script>

<script setup lang="ts">
import { useQuasar } from 'quasar'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
// UI configuration owns the menu; clicks use explicit navigation operations.
const items = (Object.keys(routePresentation) as Array<keyof RouteNamedMap>)
  .flatMap((name) => {
    const navigationItem = routePresentation[name].navigationItem
    return navigationItem ? [{ key: name, ...navigationItem }] : []
  })
  .sort((a, b) => a.order - b.order)
const activeNavigationItemKey = computed(() =>
  route.name
    ? items.find((item) => item.activeOn.includes(route.name))?.key
    : undefined
)

const $q = useQuasar()
const isDesktop = computed(() => $q.screen.width >= 1024)
</script>

<template>
  <q-drawer
    :model-value="isDesktop"
    :breakpoint="1023"
    :width="240"
    bordered
    class="app-drawer"
  >
    <div class="brand-block">
      <q-avatar
        color="primary"
        text-color="white"
        icon="auto_stories"
        rounded
        size="42px"
      />
      <div>
        <div class="text-subtitle1 text-weight-bold">智能错题本</div>
        <div class="text-caption text-grey-6">每一次回顾，都有新收获</div>
      </div>
    </div>
    <q-list padding class="q-px-sm" aria-label="主导航">
      <q-item
        v-for="item in items"
        :key="item.key"
        :active="activeNavigationItemKey === item.key"
        active-class="nav-active"
        class="q-mb-sm rounded-borders"
        @click="item.navigate()"
      >
        <q-item-section avatar>
          <q-icon :name="item.icon" />
        </q-item-section>
        <q-item-section>
          <q-item-label>{{ item.label }}</q-item-label
          ><q-item-label caption>
            {{ item.description }}
          </q-item-label>
        </q-item-section>
      </q-item>
    </q-list>
    <div class="q-pa-md">
      <q-separator /><q-item
        class="q-mt-md rounded-borders"
        @click="goSettings()"
      >
        <q-item-section avatar> <q-icon name="settings" /> </q-item-section
        ><q-item-section>设置</q-item-section>
      </q-item>
    </div>
  </q-drawer>
  <q-footer v-if="!isDesktop" bordered class="app-footer">
    <q-tabs
      :model-value="activeNavigationItemKey"
      active-color="primary"
      indicator-color="transparent"
      align="justify"
      no-caps
      class="mobile-tabs"
    >
      <q-tab
        v-for="item in items"
        :key="item.key"
        :name="item.key"
        :icon="item.icon"
        :label="item.label"
        @click="item.navigate()"
      />
    </q-tabs>
  </q-footer>
</template>
