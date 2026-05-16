<template>
  <span class="tag" :class="[type, size]">
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
interface Props {
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info'
  size?: 'small' | 'medium' | 'large'
}

const props = withDefaults(defineProps<Props>(), {
  type: 'primary',
  size: 'medium'
})
</script>

<style scoped>
.tag {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.tag.small {
  padding: 2px 6px;
  font-size: 11px;
}

.tag.medium {
  padding: 4px 8px;
  font-size: 12px;
}

.tag.large {
  padding: 6px 12px;
  font-size: 14px;
}

.tag.primary {
  background: var(--primary-light);
  color: var(--primary-color);
}

.tag.success {
  background: #e8f5e9;
  color: #43a047;
}

.tag.warning {
  background: #fff3e0;
  color: #e65100;
}

.tag.danger {
  background: #ffebee;
  color: #c62828;
}

.tag.info {
  background: #e3f2fd;
  color: #1976d2;
}
</style>