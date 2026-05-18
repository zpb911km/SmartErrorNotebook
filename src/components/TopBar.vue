<template>
  <div class="top-bar">
    <div class="top-bar-left">
      <button v-if="showBack" class="back-btn" @click="goBack">
        <span>←</span>
      </button>
      <h1 class="page-title">{{ title }}</h1>
    </div>
    <div class="top-bar-right">
      <button class="icon-btn" @click="handleSearch" v-if="showSearch">
        <span>🔍</span>
      </button>
      <button class="icon-btn" @click="handleSync" v-if="showSync">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          width="18"
          height="18"
        >
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
          <path d="M16 16h5v5" />
        </svg>
      </button>
      <button class="icon-btn" @click="handleSettings" v-if="showSettings">
        <span>⚙️</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

interface Props {
  title?: string
  showBack?: boolean
  showSearch?: boolean
  showSync?: boolean
  showSettings?: boolean
}

withDefaults(defineProps<Props>(), {
  title: '智能错题本',
  showBack: false,
  showSearch: true,
  showSync: true,
  showSettings: true
})

const router = useRouter()

const goBack = () => {
  router.back()
}

const handleSearch = () => {
  if (router.currentRoute.value.path === '/manage') {
    window.dispatchEvent(new CustomEvent('trigger-search-blink'))
  } else {
    router.push({ path: '/manage', query: { focus: 'search' } })
  }
}

const handleSync = () => {
  router.push('/sync')
}

const handleSettings = () => {
  router.push('/settings')
}
</script>

<style scoped>
.top-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 56px;
  background: var(--card-bg);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: env(safe-area-inset-top);
  padding-left: 16px;
  padding-right: 16px;
  z-index: 1000;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.top-bar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.back-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.3s;
}

.back-btn:active {
  background: var(--input-bg);
}

.page-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.top-bar-right {
  display: flex;
  gap: 8px;
}

.icon-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.3s;
}

.icon-btn:active {
  background: var(--input-bg);
}

@media (max-width: 768px) {
  .top-bar {
    height: calc(56px + env(safe-area-inset-top));
  }
}
</style>
