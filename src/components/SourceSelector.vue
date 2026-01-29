<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { getSources, createSource } from '../apis/sources';
import { Source } from '../types';
import { showInfo } from '../utils/notification';

const sources = ref<Source[]>([]);
const selectedSource = ref<Source | null>(null);
const isExpanded = ref(false);
const showAddSource = ref(false);
const newSource = ref<Omit<Source, 'id' | 'question_id'>>({
  subject_id: undefined,
  book: '',
  chapter: '',
  knowledge: '',
});

const props = defineProps<{
  currentSourceId: string;
  subjectId?: string;
}>();

const emit = defineEmits<{
  (e: 'select', source_id: string): void;
}>();

const updateSourcesForSubject = (subjectId?: string) => {
  getSources()
    .then(data => {
      if (subjectId) {
        // 过滤出属于当前科目的来源，或者没有指定科目的来源
        sources.value = data.filter(source => 
          !source.subject_id || source.subject_id === subjectId
        );
      } else {
        sources.value = data;
      }
    })
    .catch(error => {
      console.error('获取来源失败：', error);
      sources.value = [];
    });
};

// 当subjectId变化时，更新可用的来源列表
watch(
  () => props.subjectId,
  (newSubjectId) => {
    updateSourcesForSubject(newSubjectId);
  }, { immediate: true }
);

watch(
  () => props.currentSourceId,
  (newVal) => {
    if (newVal === '') {
      selectedSource.value = null;
    } else {
      const source = sources.value.find(source => source.id === newVal);
      if (source) {
        selectedSource.value = source;
      }
    }
  }
);

const handleClick = (source: Source) => {
  selectedSource.value = source;
  emit('select', source.id);
  isExpanded.value = false;
};

const handleAddSource = () => {
  // 如果有传入的subjectId，就设置到新来源中
  if (props.subjectId) {
    newSource.value.subject_id = props.subjectId;
  }
  
  createSource(newSource.value)
    .then((data) => {
      console.log('创建来源成功', data);
      showInfo('添加成功', '来源添加成功');
      
      // 直接将新来源添加到列表
      sources.value.push(data as Source);
      
      // 选择新创建的来源
      selectedSource.value = data as Source;
      showAddSource.value = false;
      isExpanded.value = false;
      emit('select', data.id);
    })
    .catch(error => {
      console.error('创建来源失败：', error);
    });
};

onMounted(() => {
  selectedSource.value = null;
});
</script>

<template>
  <div class="add-source-container" v-if="showAddSource">
    <input v-model="newSource.book" type="text" placeholder="请输入书名"></input>
    <input v-model="newSource.chapter" type="text" placeholder="请输入章节"></input>
    <input v-model="newSource.knowledge" type="text" placeholder="请输入知识点"></input>
    <button @click="handleAddSource" class="confirm">添加</button>
    <button @click="showAddSource = false" class="cancel">取消</button>
  </div>
  <div class="source-selector">
    <!-- 选择器触发按钮 -->
    <div class="selector-trigger" @click="isExpanded = !isExpanded" :class="{ 'expanded': isExpanded }">
      <span class="selected-text">
        {{ selectedSource ? `${selectedSource.book || ''} ${selectedSource.chapter || ''} ${selectedSource.knowledge || ''}`.trim() || '请选择来源' : '请选择来源' }}
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
        <div class="source-option" v-for="source in sources" :key="source.id" @click="handleClick(source)">
          <span class="option-name">
            {{ source.book ? `${source.book} ` : '' }}
            {{ source.chapter ? `${source.chapter} ` : '' }}
            {{ source.knowledge ? `${source.knowledge}` : '' }}
          </span>
        </div>
        <div class="source-option">
          <span class="option-name" @click="showAddSource = true">+ 添加新来源</span>
        </div>
        <div v-if="sources.length === 0" class="no-options">
          暂无来源数据
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.source-selector {
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

.source-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px;
  cursor: pointer;
  transition: background-color var(--transition-base);
  border-radius: calc(var(--radius-sm) - 2px);
  margin: 4px;
}

.source-option:hover {
  background-color: var(--gray-100) !important;
}

.option-name {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  flex-grow: 1;
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

.add-source-container {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: var(--z-modal);
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 20px;
  background-color: var(--card-bg);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-xl);
  min-width: 300px;
}

.add-source-container input {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-base);
  background-color: var(--input-bg);
  color: var(--text-primary);
}

.add-source-container button {
  padding: 8px 16px;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--font-size-base);
  transition: background-color var(--transition-base);
}

.add-source-container .confirm {
  background-color: var(--primary-color);
  color: var(--white);
}

.add-source-container .confirm:hover {
  background-color: var(--primary-dark);
}

.add-source-container .cancel {
  background-color: var(--gray-300);
  color: var(--gray-700);
}

.add-source-container .cancel:hover {
  background-color: var(--gray-400);
}
</style>