<template>
  <button class="btn" :class="[type, size, { block, disabled }]" :disabled="disabled" @click="handleClick">
    <span class="btn-icon" v-if="$slots.icon">
      <slot name="icon"></slot>
    </span>
    <span class="btn-text">
      <slot></slot>
    </span>
  </button>
</template>

<script setup lang="ts">
interface Props {
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'default'
  size?: 'small' | 'medium' | 'large'
  block?: boolean
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'default',
  size: 'medium',
  block: false,
  disabled: false
})

const emit = defineEmits<{
  click: []
}>()

const handleClick = () => {
  if (!props.disabled) {
    emit('click')
  }
}
</script>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
  box-sizing: border-box;
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
}

.btn.medium {
  padding: 10px 20px;
  font-size: 14px;
}

.btn.large {
  padding: 14px 28px;
  font-size: 16px;
}

.btn.block {
  width: 100%;
}

.btn.default {
  background: var(--card-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn.primary {
  background: var(--primary-color);
  color: white;
}

.btn.success {
  background: #43a047;
  color: white;
}

.btn.warning {
  background: #e65100;
  color: white;
}

.btn.danger {
  background: #c62828;
  color: white;
}

.btn:active:not(.disabled) {
  transform: scale(0.98);
}

.btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon {
  display: flex;
  align-items: center;
  font-size: 16px;
}

.btn-text {
  display: flex;
  align-items: center;
}
</style>