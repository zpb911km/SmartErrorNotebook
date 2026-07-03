<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import TopBar from './components/TopBar.vue'
import BottomNav from './components/BottomNav.vue'
import Notification from './components/Notification.vue'
import { parseImportFile } from './utils/importJson'
import { importStore } from './stores/importStore'

const route = useRoute()
const router = useRouter()

const pageTitle = computed(() => {
  return (route.meta.title as string) || '智能错题本'
})

const showTopBar = computed(() => {
  return route.path !== '/'
})

const showBottomNav = computed(() => {
  return route.path !== '/'
})

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

// 应用主题
const applyTheme = (themeValue: string) => {
  // 移除所有主题类
  document.body.classList.remove('light-theme', 'dark-theme')

  if (themeValue === 'light') {
    document.body.classList.add('light-theme')
  } else if (themeValue === 'dark') {
    document.body.classList.add('dark-theme')
  } else if (themeValue === 'system') {
    // 跟随系统
    if (
      window.matchMedia &&
      window.matchMedia('(prefers-color-scheme: dark)').matches
    ) {
      document.body.classList.add('dark-theme')
    } else {
      document.body.classList.add('light-theme')
    }
  }
}

// 初始化
onMounted(async () => {
  const savedTheme = localStorage.getItem('theme') || 'system'
  applyTheme(savedTheme)

  // 监听系统主题变化
  if (window.matchMedia) {
    window
      .matchMedia('(prefers-color-scheme: dark)')
      .addEventListener('change', () => {
        const currentTheme = localStorage.getItem('theme') || 'system'
        if (currentTheme === 'system') {
          applyTheme('system')
        }
      })
  }

  // === 文件关联处理（全局，与页面无关） ===

  // 1. 冷启动：检查 Rust State 中是否有通过文件关联传入的 URL
  try {
    const initialUrls: string[] = await invoke('opened_urls')
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
    unlistenOpened = unlisten
  } catch {
    // 桌面端不支持，静默忽略
  }
})

onUnmounted(() => {
  // 清理文件关联监听
  if (unlistenOpened) {
    unlistenOpened()
    unlistenOpened = null
  }
})
</script>

<template>
  <div id="app">
    <TopBar v-if="showTopBar" :title="pageTitle" />
    <main
      class="main-content"
      :class="{ 'with-top-bar': showTopBar, 'with-bottom-nav': showBottomNav }"
    >
      <router-view v-slot="{ Component, route: r }">
        <transition name="page-fade" mode="out-in">
          <component :is="Component" :key="r.path" />
        </transition>
      </router-view>
    </main>
    <BottomNav v-if="showBottomNav" />
    <Notification />
  </div>
</template>

<style scoped>
#app {
  min-height: 100vh;
  background-color: var(--bg-secondary);
}

.main-content {
  min-height: 100vh;
  transition: all 0.3s;
}

.main-content.with-top-bar {
  padding-top: calc(56px + env(safe-area-inset-top));
}

.main-content.with-bottom-nav {
  padding-bottom: calc(60px + env(safe-area-inset-bottom));
}
</style>
