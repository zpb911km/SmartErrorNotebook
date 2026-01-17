<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import TopBar from './components/TopBar.vue'
import BottomNav from './components/BottomNav.vue'

const route = useRoute()

const pageTitle = computed(() => {
  return route.meta.title as string || '智能错题本'
})

const showTopBar = computed(() => {
  return route.path !== '/'
})

const showBottomNav = computed(() => {
  return route.path !== '/'
})
</script>

<template>
  <div id="app">
    <TopBar v-if="showTopBar" :title="pageTitle" />
    <main class="main-content" :class="{ 'with-top-bar': showTopBar, 'with-bottom-nav': showBottomNav }">
      <router-view />
    </main>
    <BottomNav v-if="showBottomNav" />
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