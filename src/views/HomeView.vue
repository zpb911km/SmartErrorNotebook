<script setup lang="ts">
import { computed, onMounted, onScopeDispose, ref } from 'vue'

import { getLibraryStatistics } from '@/api'
import type { RouteNamedMap } from '@/router/catalog'
import type { LibraryStatisticsData } from '@/types/review'
import { createLatestTask } from '@/utils/latestTask'

const isLoading = ref(true)
const statistics = ref<LibraryStatisticsData | null>(null)
const error = ref('')

const load = createLatestTask(async (signal: AbortSignal) => {
  isLoading.value = true
  error.value = ''

  try {
    const result = await getLibraryStatistics(new Date().toISOString())
    signal.throwIfAborted()
    statistics.value = result
  } catch {
    if (signal.aborted) return
    error.value = '暂时无法读取学习数据，请重试。'
  } finally {
    if (!signal.aborted) isLoading.value = false
  }
})

onMounted(load)

onScopeDispose(load.cancel)

const actions: Array<{
  title: string
  caption: string
  icon: string
  to: { name: keyof RouteNamedMap }
}> = [
  {
    title: '添加错题',
    caption: '拍照、选图或手动记录',
    icon: 'add_photo_alternate',
    to: { name: 'question-create' }
  },
  {
    title: '整理错题',
    caption: '按科目、来源和错因查找',
    icon: 'inventory_2',
    to: { name: 'question-list' }
  },
  {
    title: '学习分析',
    caption: '了解自己的薄弱环节',
    icon: 'insights',
    to: { name: 'profile' }
  }
]

const overviewItems = computed(() => [
  {
    label: '收录错题',
    value: statistics.value?.questionTotal,
    icon: 'library_books'
  },
  {
    label: '待复习',
    value: statistics.value?.dueCount,
    icon: 'schedule'
  },
  {
    label: '累计复习',
    value: statistics.value?.totalReviews,
    icon: 'task_alt'
  }
])
</script>

<template>
  <div class="home-page">
    <div class="page-heading">
      <div class="eyebrow">温故 · 知新</div>
      <h1>让每一次错误，成为进步的阶梯</h1>
      <p>记录、整理、回顾。按自己的节奏，积累每一天的收获。</p>
    </div>
    <q-banner v-if="error" rounded class="error-banner q-mb-lg" role="alert">
      {{ error
      }}<template #action>
        <q-btn flat label="重新加载" @click="load" />
      </template>
    </q-banner>
    <div class="overview-grid overview-summary">
      <q-card
        v-for="item in overviewItems"
        :key="item.label"
        flat
        bordered
        class="overview-card"
      >
        <q-icon :name="item.icon" color="primary" size="24px" />
        <div>
          <div class="text-caption text-muted">
            {{ item.label }}
          </div>
          <q-skeleton v-if="isLoading" width="64px" height="38px" />
          <div v-else class="stat-number">
            {{ item.value ?? '—' }}
          </div>
        </div>
      </q-card>
    </div>
    <q-card flat class="review-hero">
      <q-card-section>
        <q-chip dense color="white" text-color="primary" icon="auto_stories">
          今日复习
        </q-chip>
        <h2>
          {{
            statistics?.dueCount
              ? '趁记忆还在，把知识再巩固一次'
              : '回顾旧题，发现新的理解'
          }}
        </h2>
        <p>
          {{
            statistics
              ? statistics.dueCount
                ? '有 ' +
                  statistics.dueCount +
                  ' 道错题等待复习。现在开始一轮吧。'
                : '当前没有到期题目，也可以自由回顾已有错题。'
              : '前往复习计划，查看你的学习安排。'
          }}
        </p>
        <q-btn
          unelevated
          color="white"
          text-color="primary"
          icon-right="arrow_forward"
          label="查看复习计划"
          :to="{ name: 'review-plan' }"
        />
      </q-card-section>
      <q-icon name="auto_stories" size="140px" class="hero-icon" />
    </q-card>
    <h2 class="q-mt-xl q-mb-md">从这里开始</h2>
    <div class="overview-grid">
      <q-card
        v-for="action in actions"
        :key="action.title"
        flat
        bordered
        class="quick-card"
      >
        <q-card-section>
          <q-avatar
            color="blue-1"
            text-color="primary"
            :icon="action.icon"
            rounded
          />
          <h3>{{ action.title }}</h3>
          <p>{{ action.caption }}</p>
        </q-card-section>
        <q-card-actions>
          <q-btn
            flat
            color="primary"
            :to="action.to"
            :label="action.title"
            icon-right="arrow_forward"
          />
        </q-card-actions>
      </q-card>
    </div>
  </div>
</template>
