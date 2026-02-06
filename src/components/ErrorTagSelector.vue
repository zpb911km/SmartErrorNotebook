<script setup lang="ts">
import { onMounted, ref, watch, computed } from 'vue';
import { getErrorTags } from '../apis/errorTags';
import { ErrorTags } from '../types';
import { showInfo } from '../utils/notification';

// 定义选中的标签信息类型
interface SelectedTagInfo {
  name: string;
  color: string;
}

const getRandomRGBColor = () => {
  const r = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  const g = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  const b = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  return `#${r}${g}${b}`;
};

const errorTags = ref<ErrorTags[]>([]);
const selectedTags = ref<SelectedTagInfo[]>([]);
const isExpanded = ref(false);
const showAddErrorTag = ref(false);
const newErrorTag = ref<Omit<ErrorTags, 'id' | 'question_id'>>({
  name: '',
  color: getRandomRGBColor(),
});

const props = defineProps<{
  currentTags?: SelectedTagInfo[];
}>();

const emit = defineEmits<{
  (e: 'select', tags: SelectedTagInfo[]): void;
}>();

// 显示文本
const displayText = computed(() => {
  if (selectedTags.value.length === 0) {
    return '请选择错因';
  } else if (selectedTags.value.length === 1) {
    return selectedTags.value[0].name;
  } else {
    return `已选择 ${selectedTags.value.length} 个错因`;
  }
});

watch(
  () => props.currentTags,
  (newVal) => {
    if (newVal) {
      selectedTags.value = [...newVal];
    } else {
      selectedTags.value = [];
    }
  },
  { immediate: true }
);

const handleClick = (errorTag: ErrorTags, event: Event) => {
  event.stopPropagation();
  const index = selectedTags.value.findIndex(tag => tag.name === errorTag.name && tag.color === errorTag.color);
  if (index > -1) {
    // 取消选择
    selectedTags.value.splice(index, 1);
  } else {
    // 添加选择（只保存name和color）
    selectedTags.value.push({
      name: errorTag.name,
      color: errorTag.color,
    });
  }
  emit('select', [...selectedTags.value]);
};

const handleAddErrorTag = () => {
  // 不再调用API创建标签，只是添加到已选列表
  console.log('添加错因标签', newErrorTag.value);
  showInfo('添加成功', '错因标签添加成功');
  
  // 添加到已选列表
  selectedTags.value.push({
    name: newErrorTag.value.name,
    color: newErrorTag.value.color,
  });
  
  emit('select', [...selectedTags.value]);
  showAddErrorTag.value = false;
  
  // 重置新标签表单
  newErrorTag.value = {
    name: '',
    color: getRandomRGBColor(),
  };
};

// 检查标签是否被选中
const isTagSelected = (errorTag: ErrorTags) => {
  return selectedTags.value.some(tag => tag.name === errorTag.name && tag.color === errorTag.color);
};

// 移除标签
const removeTag = (index: number) => {
  selectedTags.value.splice(index, 1);
  emit('select', [...selectedTags.value]);
};

onMounted(() => {
  getErrorTags()
    .then(data => {
      errorTags.value = data;
    })
    .catch(error => {
      console.error('获取错因标签失败：', error);
    });
});
</script>

<template>
  <div class="error-tag-selector">
    <!-- 已选标签展示 -->
    <div class="selected-tags" v-if="selectedTags.length > 0">
      <div
        class="tag-chip"
        v-for="(tag, index) in selectedTags"
        :key="`${tag.name}-${tag.color}`"
        :style="{ backgroundColor: tag.color }"
      >
        {{ tag.name }}
        <span class="remove-tag" @click.stop="removeTag(index)">✕</span>
      </div>
    </div>

    <!-- 选择器触发按钮 -->
    <div class="selector-trigger" @click="isExpanded = !isExpanded" :class="{ 'expanded': isExpanded }">
      <span class="selected-text">
        {{ displayText }}
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
        <div
          class="error-tag-option"
          v-for="errorTag in errorTags"
          :key="errorTag.id"
          @click="handleClick(errorTag, $event)"
          :class="{ 'selected': isTagSelected(errorTag) }"
        >
          <div class="option-left">
            <span class="checkbox" :class="{ 'checked': isTagSelected(errorTag) }"></span>
            <span class="option-name">{{ errorTag.name }}</span>
          </div>
          <div class="color-indicator" :style="{ backgroundColor: errorTag.color }"></div>
        </div>
        <div  v-if="!showAddErrorTag" class="error-tag-option" @click="showAddErrorTag = true">
          <span class="option-name">+ 添加新错因</span>
        </div>
        <div class="add-error-tag-container" v-if="showAddErrorTag">
          <input v-model="newErrorTag.name" type="text" placeholder="请输入错因名称"></input>
          <input v-model="newErrorTag.color" type="color" title="选择颜色" :style="{ backgroundColor: newErrorTag.color }"></input>
          <button @click="handleAddErrorTag" class="confirm">添加</button>
          <button @click="showAddErrorTag = false" class="cancel">取消</button>
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

.selected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 500;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.remove-tag {
  margin-left: 6px;
  cursor: pointer;
  opacity: 0.8;
  transition: opacity 0.2s;
}

.remove-tag:hover {
  opacity: 1;
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
  max-height: 250px;
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

.error-tag-option.selected {
  background-color: var(--primary-light) !important;
}

.option-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-grow: 1;
}

.checkbox {
  width: 18px;
  height: 18px;
  border: 2px solid var(--gray-400);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.checkbox.checked {
  background-color: var(--primary-color);
  border-color: var(--primary-color);
}

.checkbox.checked::after {
  content: '✓';
  color: white;
  font-size: 12px;
  font-weight: bold;
}

.option-name {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
}

.color-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  margin-left: 10px;
  border: 1px solid var(--white);
  flex-shrink: 0;
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
  max-height: 250px;
}

.slide-down-leave-active {
  transition: all var(--transition-base);
  max-height: 250px;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
  max-height: 0;
}

.add-error-tag-container {
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