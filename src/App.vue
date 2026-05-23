<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import TopBar from './components/TopBar.vue'
import BottomNav from './components/BottomNav.vue'
import Notification from './components/Notification.vue'

const route = useRoute()

const pageTitle = computed(() => {
  return (route.meta.title as string) || '智能错题本'
})

const showTopBar = computed(() => {
  return route.path !== '/'
})

const showBottomNav = computed(() => {
  return route.path !== '/'
})

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

// 初始化主题
onMounted(() => {
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
})
</script>

<template>
  <div id="app">
    <TopBar v-if="showTopBar" :title="pageTitle" />
    <main
      class="main-content"
      :class="{ 'with-top-bar': showTopBar, 'with-bottom-nav': showBottomNav }"
    >
      <router-view />
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
