<template>
  <div class="notification-container">
    <transition-group
      name="notification"
      tag="div"
      class="notification-wrapper"
    >
      <div
        v-for="notification in notifications"
        :key="notification.id"
        :class="['notification', `notification--${notification.type}`]"
      >
        <div class="notification__icon">
          <Icon v-if="notification.type === 'info'" name="info" :size="20" />
          <Icon
            v-if="notification.type === 'success'"
            name="circle-check"
            :size="20"
          />
          <Icon
            v-if="notification.type === 'warning'"
            name="triangle-alert"
            :size="20"
          />
          <Icon
            v-if="notification.type === 'error'"
            name="circle-x"
            :size="20"
          />
          <Icon v-if="notification.type === 'debug'" name="wrench" :size="20" />
        </div>
        <div class="notification__content">
          <div class="notification__title">{{ notification.title }}</div>
          <div class="notification__message">{{ notification.message }}</div>
        </div>
        <button
          class="notification__close"
          @click="removeNotification(notification.id)"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
          </svg>
        </button>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

export interface Notification {
  id: string
  type: 'info' | 'success' | 'warning' | 'error' | 'debug'
  title: string
  message: string
  duration?: number
}

const notifications = ref<Notification[]>([])

// 生成唯一ID
const generateId = () => {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// 添加通知
const addNotification = (
  type: Notification['type'],
  title: string,
  message: string,
  duration = 5000
) => {
  const id = generateId()
  const notification: Notification = {
    id,
    type,
    title,
    message,
    duration
  }

  notifications.value.push(notification)

  // 自动移除通知
  if (duration > 0) {
    setTimeout(() => {
      removeNotification(id)
    }, duration)
  }
}

// 移除通知
const removeNotification = (id: string) => {
  const index = notifications.value.findIndex(
    (notification) => notification.id === id
  )
  if (index !== -1) {
    notifications.value.splice(index, 1)
  }
}

// 暴露方法给外部使用
defineExpose({
  addNotification,
  removeNotification
})
</script>

<style scoped>
.notification-container {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  max-width: 420px;
  width: 90vw;
}

.notification-wrapper {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.notification {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  background: var(--card-bg);
  min-width: 300px;
  max-width: 100%;
  animation: slideIn 0.3s ease-out;
  border-left: 4px solid var(--primary-color);
}

.notification--info {
  border-left-color: var(--info-color);
}

.notification--success {
  border-left-color: var(--success-color);
}

.notification--warning {
  border-left-color: var(--warning-color);
}

.notification--error {
  border-left-color: var(--danger-color);
}

.notification--debug {
  border-left-color: var(--secondary-color);
}

.notification__icon {
  margin-right: 12px;
  display: flex;
  align-items: center;
}

.notification__content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.notification__title {
  font-weight: var(--font-weight-semibold);
  margin-bottom: var(--spacing-sm);
  color: var(--text-primary);
}

.notification__message {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: var(--line-height-normal);
}

.notification__close {
  cursor: pointer;
  background: var(--bg-tertiary);
  border: none;
  padding: 4px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all var(--transition-fast);
}

.notification__close:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

/* 过渡动画 */
.notification-enter-active {
  transition: all 0.3s ease;
  opacity: 1;
}

.notification-leave-active {
  transition: all 0.3s ease;
  opacity: 1;
}

.notification-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}

.notification-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
