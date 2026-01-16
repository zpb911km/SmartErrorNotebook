<template>
  <div class="card" :class="{ clickable: clickable }" @click="handleClick">
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
interface Props {
  clickable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  clickable: false
})

const emit = defineEmits<{
  click: []
}>()

const handleClick = () => {
  if (props.clickable) {
    emit('click')
  }
}
</script>

<style scoped>
.card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  transition: all 0.3s;
}

.card.clickable {
  cursor: pointer;
}

.card.clickable:active {
  transform: scale(0.98);
}
</style>