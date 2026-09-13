<script setup lang="ts">
import { useQuasar } from 'quasar'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const q = useQuasar()
const desktop = computed(() => q.screen.width >= 1024)
const active = computed(() =>
  route.path.startsWith('/manage-detail')
    ? '/manage'
    : route.path === '/review-detail'
      ? '/review'
      : route.path
)
const items = [
  {
    path: '/home',
    label: '首页',
    icon: 'space_dashboard',
    description: '学习概览'
  },
  {
    path: '/add',
    label: '添加',
    icon: 'add_circle_outline',
    description: '记录新的收获'
  },
  {
    path: '/manage',
    label: '管理',
    icon: 'inventory_2',
    description: '整理错题与知识'
  },
  {
    path: '/review',
    label: '复习',
    icon: 'auto_stories',
    description: '让知识更牢固'
  },
  {
    path: '/stats',
    label: '我的',
    icon: 'insights',
    description: '查看学习进展'
  }
]
</script>

<template>
  <q-drawer
    :model-value="desktop"
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
        :key="item.path"
        :to="item.path"
        :active="active === item.path"
        active-class="nav-active"
        class="q-mb-sm rounded-borders"
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
      <q-separator /><q-item to="/settings" class="q-mt-md rounded-borders">
        <q-item-section avatar> <q-icon name="settings" /> </q-item-section
        ><q-item-section>设置</q-item-section>
      </q-item>
    </div>
  </q-drawer>
  <q-footer v-if="!desktop" bordered class="app-footer">
    <q-tabs
      :model-value="active"
      active-color="primary"
      indicator-color="transparent"
      align="justify"
      no-caps
      class="mobile-tabs"
    >
      <q-tab
        v-for="item in items"
        :key="item.path"
        :name="item.path"
        :icon="item.icon"
        :label="item.label"
        @click="$router.push(item.path)"
      />
    </q-tabs>
  </q-footer>
</template>
