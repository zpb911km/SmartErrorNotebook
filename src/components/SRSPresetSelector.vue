<script setup lang="ts">
import { ref, watch } from 'vue';

interface SRSDataPreset {
  id: string;
  name: string;
  difficulty: number; // 难度 (0-1)
  mastery: number; // 掌握程度 (0-1)
}

const srsPresets = ref<SRSDataPreset[]>([
  { id: '1', name: '初始难度', difficulty: 0.5, mastery: 0.5 },
  { id: '2', name: '较难', difficulty: 0.7, mastery: 0.3 },
  { id: '3', name: '较易', difficulty: 0.3, mastery: 0.7 },
  { id: '4', name: '困难', difficulty: 0.9, mastery: 0.1 },
  { id: '5', name: '熟练', difficulty: 0.1, mastery: 0.9 },
]);

const selectedPreset = ref<SRSDataPreset | null>(null);
const isExpanded = ref(false);

const props = defineProps<{
  currentPresetId: string;
}>();

const emit = defineEmits<{
  (e: 'select', preset: SRSDataPreset): void;
}>();

watch(
  () => props.currentPresetId,
  (newVal) => {
    if (newVal === '') {
      selectedPreset.value = null;
    } else {
      const preset = srsPresets.value.find(preset => preset.id === newVal);
      if (preset) {
        selectedPreset.value = preset;
      }
    }
  }
);

const handleClick = (preset: SRSDataPreset) => {
  selectedPreset.value = preset;
  emit('select', preset);
  isExpanded.value = false;
};
</script>

<template>
  <div class="srs-preset-selector">
    <!-- 选择器触发按钮 -->
    <div class="selector-trigger" @click="isExpanded = !isExpanded" :class="{ 'expanded': isExpanded }">
      <span class="selected-text">
        {{ selectedPreset ? selectedPreset.name : '请选择SRS预设' }}
      </span>
      <svg class="arrow-icon" :class="{ 'rotated': isExpanded }" xmlns="http://www.w3.org/2000/svg" width="16"
        height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
        stroke-linejoin="round">
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </div>

    <!-- 下拉选项 -->
    <transition name="slide-down">
      <div v-if="isExpanded" class="dropdown-container">
        <div class="preset-option" v-for="preset in srsPresets" :key="preset.id" @click="handleClick(preset)">
          <span class="option-name">{{ preset.name }}</span>
          <div class="preset-info">
            <span>难度: {{ Math.round(preset.difficulty * 100) }}%</span>
            <span>掌握: {{ Math.round(preset.mastery * 100) }}%</span>
          </div>
        </div>
        <div v-if="srsPresets.length === 0" class="no-options">
          暂无SRS预设数据
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.srs-preset-selector {
  position: relative;
  width: 100%;
  font-family: var(--font-family-base);
}

.selector-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 15px;
  background-color: var(--card-bg);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-base);
  min-height: 40px;
}

.selector-trigger:hover {
  border-color: var(--gray-400);
  box-shadow: var(--shadow-sm);
}

.selector-trigger.expanded {
  border-color: var(--gray-600);
  box-shadow: var(--shadow-md);
}

.selected-text {
  color: var(--text-primary);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.arrow-icon {
  margin-left: 10px;
  transition: transform var(--transition-base);
  color: var(--gray-500);
}

.arrow-icon.rotated {
  transform: rotate(180deg);
}

.dropdown-container {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: var(--z-dropdown);
  margin-top: 5px;
  background-color: var(--card-bg);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  max-height: 200px;
  overflow-y: auto;
}

.preset-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px;
  cursor: pointer;
  transition: background-color var(--transition-base);
  border-radius: calc(var(--radius-sm) - 2px);
  margin: 4px;
}

.preset-option:hover {
  background-color: var(--gray-100) !important;
}

.option-name {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  flex-grow: 1;
}

.preset-info {
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.preset-info span {
  line-height: 1.2;
}

.no-options {
  padding: 15px;
  text-align: center;
  color: var(--gray-500);
  font-style: italic;
  font-size: var(--font-size-base);
}

/* 下拉动画 */
.slide-down-enter-active {
  transition: all var(--transition-base);
  max-height: 200px;
}

.slide-down-leave-active {
  transition: all var(--transition-base);
  max-height: 200px;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
  max-height: 0;
}
</style>