<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { getSubjects } from '../apis/subjects';
import { Subject } from '../types';

const subjects = ref<Subject[]>([]);
const selectedSubject = ref<Subject | null>(null);
const isExpanded = ref(false);

const emit = defineEmits<{
  (e: 'select', subject_id: string): void;
}>();

const handleClick = (subject: Subject) => {
  selectedSubject.value = subject;
  emit('select', subject.id);
  isExpanded.value = false;
};

onMounted(() => {
  getSubjects()
    .then(data => {
      subjects.value = data;
    })
    .catch(error => {
      console.error('获取科目失败：', error);
    });
  selectedSubject.value = null;
});
</script>

<template>
  <div class="subject-selector">
    <!-- 选择器触发按钮 -->
    <div 
      class="selector-trigger" 
      @click="isExpanded = !isExpanded"
      :class="{ 'expanded': isExpanded }"
    >
      <span class="selected-text">
        {{ selectedSubject ? selectedSubject.name : '请选择科目' }}
      </span>
      <svg 
        class="arrow-icon" 
        :class="{ 'rotated': isExpanded }" 
        xmlns="http://www.w3.org/2000/svg" 
        width="16" 
        height="16" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        stroke-width="2" 
        stroke-linecap="round" 
        stroke-linejoin="round"
      >
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </div>
    
    <!-- 下拉选项 -->
    <transition name="slide-down">
      <div v-if="isExpanded" class="dropdown-container">
        <div
          class="subject-option"
          v-for="subject in subjects"
          :key="subject.id"
          @click="handleClick(subject)"
        >
          <span class="option-name">{{ subject.name }}</span>
          <div 
            class="color-indicator" 
            :style="{ backgroundColor: subject.color }"
          ></div>
        </div>
        <div 
          v-if="subjects.length === 0" 
          class="no-options"
        >
          暂无科目数据
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.subject-selector {
  position: relative;
  width: 200px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

.selector-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 15px;
  background-color: #ffffff;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: 40px;
}

.selector-trigger:hover {
  border-color: #cbd5e0;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.05);
}

.selector-trigger.expanded {
  border-color: #a0aec0;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.selected-text {
  color: #2d3748;
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.arrow-icon {
  margin-left: 10px;
  transition: transform 0.2s ease;
  color: #718096;
}

.arrow-icon.rotated {
  transform: rotate(180deg);
}

.dropdown-container {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 10;
  margin-top: 5px;
  background-color: #ffffff;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  max-height: 200px;
  overflow-y: auto;
}

.subject-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px;
  cursor: pointer;
  transition: background-color 0.2s ease;
  border-radius: 6px;
  margin: 4px;
}

.subject-option:hover {
  background-color: #f7fafc !important;
}

.option-name {
  font-size: 14px;
  font-weight: 500;
  flex-grow: 1;
}

.color-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  margin-left: 10px;
  border: 1px solid rgba(255, 255, 255, 0.5);
}

.no-options {
  padding: 15px;
  text-align: center;
  color: #a0aec0;
  font-style: italic;
  font-size: 14px;
}

/* 下拉动画 */
.slide-down-enter-active {
  transition: all 0.2s ease;
  max-height: 200px;
}

.slide-down-leave-active {
  transition: all 0.2s ease;
  max-height: 200px;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
  max-height: 0;
}
</style>