<template>
  <div class="notification-container">
    <transition-group name="notification" tag="div" class="notification-wrapper">
      <div
        v-for="notification in notifications"
        :key="notification.id"
        :class="['notification', `notification--${notification.type}`]"
      >
        <div class="notification__icon">
          <span v-if="notification.type === 'info'">ℹ️</span>
          <span v-if="notification.type === 'success'">✅</span>
          <span v-if="notification.type === 'warning'">⚠️</span>
          <span v-if="notification.type === 'error'">❌</span>
          <span v-if="notification.type === 'debug'">🔧</span>
        </div>
        <div class="notification__content">
          <div class="notification__title">{{ notification.title }}</div>
          <div class="notification__message">{{ notification.message }}</div>
        </div>
        <div class="notification__close" @click="removeNotification(notification.id)">×</div>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error' | 'debug';
  title: string;
  message: string;
  duration?: number;
}

const notifications = ref<Notification[]>([]);

// 生成唯一ID
const generateId = () => {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
};

// 添加通知
const addNotification = (type: Notification['type'], title: string, message: string, duration = 5000) => {
  const id = generateId();
  const notification: Notification = {
    id,
    type,
    title,
    message,
    duration
  };
  
  notifications.value.push(notification);
  
  // 自动移除通知
  if (duration > 0) {
    setTimeout(() => {
      removeNotification(id);
    }, duration);
  }
};

// 移除通知
const removeNotification = (id: string) => {
  const index = notifications.value.findIndex(notification => notification.id === id);
  if (index !== -1) {
    notifications.value.splice(index, 1);
  }
};

// 暴露方法给外部使用
defineExpose({
  addNotification,
  removeNotification
});
</script>

<style scoped>
.notification-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;
  max-width: 400px;
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
  animation: slideInRight 0.3s ease-out;
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

.notification--info {
  border-left-color: #2196f3;
}

.notification--success {
  border-left-color: #4caf50;
}

.notification--warning {
  border-left-color: #ff9800;
}

.notification--error {
  border-left-color: #f44336;
}

.notification--debug {
  border-left-color: #9c27b0;
}

.notification__icon {
  font-size: 1.2em;
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
  font-size: 1.5em;
  padding: 0 5px;
  color: var(--text-disabled);
  user-select: none;
}

.notification__close:hover {
  color: var(--text-secondary);
}

/* 过渡动画 */
.notification-enter-active, .notification-leave-active {
  transition: all 0.3s ease;
  transform: translateX(0);
  opacity: 1;
}

.notification-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.notification-leave-to {
  transform: translateX(100%);
  opacity: 0;
  position: absolute;
  width: 100%;
}

@keyframes slideInRight {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>