<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { getOpenedUrls } from './api/platformExceptions'
import AppNavigation from './components/AppNavigation.vue'
import { initializeTheme } from './composables/useTheme'
import { importStore } from './stores/importStore'
import { parseImportFile } from './utils/importJson'

const route = useRoute()
const router = useRouter()

const pageTitle = computed(() => {
  return (route.meta.title as string) || '智能错题本'
})

const parentPath = computed(() =>
  route.path.startsWith('/manage-detail')
    ? '/manage'
    : route.path === '/review-detail'
      ? '/review'
      : '/home'
)
const showBack = computed(() =>
  /detail|settings|sync|markdown-test/.test(route.path)
)
function search() {
  if (route.path === '/manage')
    window.dispatchEvent(new CustomEvent('focus-library-search'))
  else router.push({ path: '/manage', query: { focus: 'search' } })
}
const themeLifecycle = initializeTheme()

/** 统一处理文件关联传入的 URL */
const handleOpenedUrl = async (url: string) => {
  try {
    // 使用 Tauri fs plugin 读取文件内容
    // 桌面端支持 file:// 和绝对路径，Android 支持 content:// URI
    const { readTextFile } = await import('@tauri-apps/plugin-fs')
    const content = await readTextFile(url)

    const result = parseImportFile(content)
    if (result.error) {
      console.warn('文件关联导入解析失败:', result.error)
      return
    }

    // 解析成功，存入全局 store
    importStore.pendingData = {
      questions: result.questions.map((q) => ({
        prompt: q.prompt,
        answer: q.answer || '',
        analysis: q.analysis || ''
      })),
      version: result.version
    }

    // 跳转到管理页面（如果不在那里），触发导入弹窗
    if (route.path !== '/manage') {
      router.push('/manage?import=1')
    } else {
      // 已经在管理页面，用 query 的变化触发重新检查
      router.replace('/manage?import=1')
    }
  } catch (e) {
    console.error('处理文件关联导入失败:', e)
  }
}

/** 取消文件关联事件监听 */
let unlistenOpened: (() => void) | null = null
let disposed = false

onMounted(async () => {
  // === 文件关联处理（全局，与页面无关） ===

  // 1. 冷启动：检查 Rust State 中是否有通过文件关联传入的 URL
  try {
    const initialUrls: string[] = await getOpenedUrls()
    if (initialUrls.length > 0) {
      await handleOpenedUrl(initialUrls[0])
    }
  } catch {
    // 桌面端不支持，静默忽略
  }

  // 2. 热启动：监听应用被文件关联唤起的事件
  try {
    const unlisten = await listen<string[]>('opened', async (event) => {
      if (event.payload.length > 0) {
        await handleOpenedUrl(event.payload[0])
      }
    })
    if (disposed) unlisten()
    else unlistenOpened = unlisten
  } catch {
    // 桌面端不支持，静默忽略
  }
})

onUnmounted(() => {
  disposed = true
  themeLifecycle.dispose()
  // 清理文件关联监听
  if (unlistenOpened) {
    unlistenOpened()
    unlistenOpened = null
  }
})
</script>

<template>
  <q-layout view="hHh Lpr lFf" class="notebook-layout">
    <q-header bordered class="app-header">
      <q-toolbar class="app-toolbar">
        <q-btn
          v-if="showBack"
          flat
          round
          icon="arrow_back"
          :to="parentPath"
          aria-label="返回"
        />
        <q-toolbar-title class="text-weight-bold">
          {{ pageTitle }}
        </q-toolbar-title>
        <q-btn flat round icon="search" aria-label="搜索错题" @click="search">
          <q-tooltip>搜索错题</q-tooltip>
        </q-btn>
        <q-btn flat round icon="settings" to="/settings" aria-label="设置">
          <q-tooltip>设置</q-tooltip>
        </q-btn>
      </q-toolbar>
    </q-header>
    <AppNavigation />
    <q-page-container>
      <q-page class="app-page">
        <router-view />
      </q-page>
    </q-page-container>
  </q-layout>
</template>
