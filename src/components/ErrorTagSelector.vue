<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { getErrorTags, createErrorTag } from '../apis/errorTags';
import { ErrorTags } from '../types';
import { showInfo } from '../utils/notification';

const getRandomRGBColor = () => {
  const r = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  const g = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  const b = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  return `#${r}${g}${b}`;
};

const errorTags = ref<ErrorTags[]>([]);
const selectedErrorTag = ref<ErrorTags | null>(null);
const isExpanded = ref(false);
const showAddErrorTag = ref(false);
const newErrorTag = ref<Omit<ErrorTags, 'id' | 'question_id'>>({
  name: '',
  color: getRandomRGBColor(),
});

const props = defineProps<{
  currentErrorTagId: string;
}>();

const emit = defineEmits<{
  (e: 'select', error_tag_id: string): void;
}>();

watch(
  () => props.currentErrorTagId,
  (newVal) => {
    if (newVal === '') {
      selectedErrorTag.value = null;
    } else {
      const tag = errorTags.value.find(tag => tag.id === newVal);
      if (tag) {
        selectedErrorTag.value = tag;
      }
    }
  }
);

const handleClick = (errorTag: ErrorTags) => {
  selectedErrorTag.value = errorTag;
  emit('select', errorTag.id);
  isExpanded.value = false;
};

const handleAddErrorTag = () => {
  createErrorTag(newErrorTag.value)
    .then((data) => {
      console.log('创建错因标签成功', data);
      showInfo('添加成功', '错因标签添加成功');
      errorTags.value.push(data);
      selectedErrorTag.value = data;
      showAddErrorTag.value = false;
      isExpanded.value = false;
      emit('select', data.id);
    })
    .catch(error => {
      console.error('创建错因标签失败：', error);
    });
};

onMounted(() => {
  getErrorTags()
    .then(data => {
      errorTags.value = data;
    })
    .catch(error => {
      console.error('获取错因标签失败：', error);
    });
  selectedErrorTag.value = null;
});
</script>

<template>
  <div class="add-error-tag-container" v-if="showAddErrorTag">
    <input v-model="newErrorTag.name" type="text" placeholder="请输入错因名称"></input>
    <input v-model="newErrorTag.color" type="color" title="选择颜色" :style="{ backgroundColor: newErrorTag.color }"></input>
    <button @click="handleAddErrorTag" class="confirm">添加</button>
    <button @click="showAddErrorTag = false" class="cancel">取消</button>
  </div>
  <div class="error-tag-selector">
    <!-- 选择器触发按钮 -->
    <div class="selector-trigger" @click="isExpanded = !isExpanded" :class="{ 'expanded': isExpanded }">
      <span class="selected-text">
        {{ selectedErrorTag ? selectedErrorTag.name : '请选择错因' }}
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
        <div class="error-tag-option" v-for="errorTag in errorTags" :key="errorTag.id" @click="handleClick(errorTag)">
          <span class="option-name">{{ errorTag.name }}</span>
          <div class="color-indicator" :style="{ backgroundColor: errorTag.color }"></div>
        </div>
        <div class="error-tag-option">
          <span class="option-name" @click="showAddErrorTag = true">+ 添加新错因</span>
        </div>
        <div v-if="errorTags.length === 0" class="no-options">
          暂无错因标签数据
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.error-tag-selector {
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

.error-tag-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px;
  cursor: pointer;
  transition: background-color var(--transition-base);
  border-radius: calc(var(--radius-sm) - 2px);
  margin: 4px;
}

.error-tag-option:hover {
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

.add-error-tag-container {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
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

.add-error-tag-container input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-base);
  background-color: var(--input-bg);
  color: var(--text-primary);
}

.add-error-tag-container button {
  padding: 8px 16px;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--font-size-base);
  transition: background-color var(--transition-base);
}

.add-error-tag-container .confirm {
  background-color: var(--primary-color);
  color: var(--white);
}

.add-error-tag-container .confirm:hover {
  background-color: var(--primary-dark);
}

.add-error-tag-container .cancel {
  background-color: var(--gray-300);
  color: var(--gray-700);
}

.add-error-tag-container .cancel:hover {
  background-color: var(--gray-400);
}
</style>