<template>
  <nav class="bottom-nav">
    <router-link
      v-for="item in navItems"
      :key="item.path"
      :to="item.path"
      class="nav-item"
      :class="{ active: isActive(item.path) }"
    >
      <span class="nav-icon">{{ item.icon }}</span>
      <span class="nav-label">{{ item.label }}</span>
    </router-link>
  </nav>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()

const navItems = [
  { path: '/home', label: '首页', icon: '🏠' },
  { path: '/add', label: '添加', icon: '➕' },
  { path: '/manage', label: '管理', icon: '📋' },
  { path: '/review', label: '复习', icon: '📖' },
  { path: '/stats', label: '我的', icon: '👤' }
]

const isActive = (path: string) => {
  return route.path === path
}
</script>

<style scoped>
.bottom-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 60px;
  background: var(--card-bg);
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: space-around;
  align-items: center;
  z-index: 1000;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.08);
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-decoration: none;
  color: var(--text-secondary);
  transition: all 0.3s;
  padding: 8px 12px;
  border-radius: 8px;
}

.nav-item.active {
  color: var(--primary-color);
}

.nav-icon {
  font-size: 20px;
}

.nav-label {
  font-size: 11px;
  font-weight: 500;
}

.nav-item:active {
  transform: scale(0.95);
}
</style>
