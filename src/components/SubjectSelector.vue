<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { createSubject, getSubjects } from '../apis/subjects';
import { Subject } from '../types';
import { showInfo } from '../utils/notification';

const getRandomRGBColor = () => {
  const r = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  const g = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  const b = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  return `#${r}${g}${b}`;
};

const subjects = ref<Subject[]>([]);
const selectedSubject = ref<Subject | null>(null);
const isExpanded = ref(false);
const showAddSubject = ref(false);
const newSubject = ref<Subject>({
  id: '',
  name: '',
  color: getRandomRGBColor(),
});

const props = defineProps<{
  currentSubjectId: string;
}>();

const emit = defineEmits<{
  (e: 'select', subject_id: string): void;
}>();

watch(
  () => props.currentSubjectId,
  (newVal) => {
    if (newVal === '') {
      selectedSubject.value = null;
    }
    const subject = subjects.value.find(subject => subject.id === newVal);
    if (subject) {
      selectedSubject.value = subject;
    }
  }
)

watch(
  () => showAddSubject,
  () => {
    newSubject.value = {
      id: '',
      name: '',
      color: getRandomRGBColor(),
    };
  }
)

const handleClick = (subject: Subject) => {
  selectedSubject.value = subject;
  emit('select', subject.id);
  isExpanded.value = false;
};

const handleAddSubject = () => {
  createSubject(newSubject.value)
    .then((data) => {
      console.log(data)
      showInfo('添加成功', '科目添加成功');
      subjects.value.push(newSubject.value);
      selectedSubject.value = newSubject.value;
      showAddSubject.value = false;
      isExpanded.value = false;
      emit('select', newSubject.value.id);
    })
    .catch(error => {
      console.error('创建科目失败：', error);
    });
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
    <div class="selector-trigger" @click="isExpanded = !isExpanded" :class="{ 'expanded': isExpanded }">
      <span class="selected-text">
        {{ selectedSubject ? selectedSubject.name : '请选择科目' }}
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
        <div class="subject-option" v-for="subject in subjects" :key="subject.id" @click="handleClick(subject)">
          <span class="option-name">{{ subject.name }}</span>
          <div class="color-indicator" :style="{ backgroundColor: subject.color }"></div>
        </div>
        <div v-if="!showAddSubject" class="subject-option" @click="showAddSubject = true">
          <span class="option-name">+ 添加新科目</span>
        </div>
        <div class="add-subject-container" v-if="showAddSubject">
          <input v-model="newSubject.name" type="text" placeholder="请输入科目名称"></input>
          <input v-model="newSubject.color" type="color" title="选择颜色" :style="{ backgroundColor: newSubject.color }"></input>
          <button @click="handleAddSubject" class="confirm">添加</button>
          <button @click="showAddSubject = false" class="cancel">取消</button>
        </div>
        <div v-if="subjects.length === 0" class="no-options">
          暂无科目数据
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.subject-selector {
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

.subject-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px;
  cursor: pointer;
  transition: background-color var(--transition-base);
  border-radius: calc(var(--radius-sm) - 2px);
  margin: 4px;
}

.subject-option:hover {
  background-color: var(--gray-100) !important;
}

.option-name {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  flex-grow: 1;
}

.color-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  margin-left: 10px;
  border: 1px solid var(--white);
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

.add-subject-container {
  width: 50%;
  z-index: var(--z-modal);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px;
  background-color: var(--card-bg);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-xl);
  min-width: 300px;
}

.add-subject-container input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-base);
  background-color: var(--input-bg);
  color: var(--text-primary);
}

.add-subject-container button {
  padding: 8px 16px;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--font-size-base);
  transition: background-color var(--transition-base);
}

.add-subject-container .confirm {
  background-color: var(--primary-color);
  color: var(--white);
}

.add-subject-container .confirm:hover {
  background-color: var(--primary-dark);
}

.add-subject-container .cancel {
  background-color: var(--gray-300);
  color: var(--gray-700);
}

.add-subject-container .cancel:hover {
  background-color: var(--gray-400);
}
</style>