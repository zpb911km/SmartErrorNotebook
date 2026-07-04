<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { createSubject, getSubjects } from '../apis/subjects'
import { Subject } from '../types'
import { showInfo } from '../utils/notification'

const getRandomRGBColor = () => {
  const r = Math.floor(Math.random() * 256)
    .toString(16)
    .padStart(2, '0')
  const g = Math.floor(Math.random() * 256)
    .toString(16)
    .padStart(2, '0')
  const b = Math.floor(Math.random() * 256)
    .toString(16)
    .padStart(2, '0')
  return `#${r}${g}${b}`
}

const subjects = ref<Subject[]>([])
const selectedSubject = ref<Subject | null>(null)
const isExpanded = ref(false)
const showAddSubject = ref(false)
const newSubject = ref<Subject>({
  id: '',
  name: '',
  color: getRandomRGBColor()
})

const props = defineProps<{
  modelValue: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'select', subject_id: string): void
}>()
// 添加一个监听器监听disable，如果disable为true，则收起
watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) {
      isExpanded.value = false
    }
  }
)
watch(
  () => props.modelValue,
  (newVal) => {
    console.log('modelValue changed:', newVal)
    if (!newVal || newVal === '') {
      selectedSubject.value = null
      return
    }
    // 如果科目列表已加载，立即查找
    if (subjects.value.length > 0) {
      const subject = subjects.value.find((s) => s.id === newVal)
      if (subject) {
        selectedSubject.value = subject
        console.log('找到对应科目:', subject.name)
      }
    }
  },
  { immediate: true }
)

watch(
  () => showAddSubject,
  () => {
    newSubject.value = {
      id: '',
      name: '',
      color: getRandomRGBColor()
    }
  }
)

const handleClick = (subject: Subject) => {
  selectedSubject.value = subject
  emit('select', subject.id)
  isExpanded.value = false
}

const handleAddSubject = () => {
  createSubject(newSubject.value)
    .then((data) => {
      console.log(data)
      showInfo('添加成功', '科目添加成功')
      subjects.value.push({ ...data })
      selectedSubject.value = { ...data }
      showAddSubject.value = false
      isExpanded.value = false
      newSubject.value = {
        id: '',
        name: '',
        color: getRandomRGBColor()
      }
      emit('select', selectedSubject.value.id)
    })
    .catch((error) => {
      console.error('创建科目失败：', error)
    })
}

watch(isExpanded, (newVal) => {
  if (newVal) {
    getSubjects()
      .then((data) => {
        subjects.value = data
        // 加载完科目列表后，根据modelValue重新设置选中状态
        if (props.modelValue) {
          const found = data.find((s) => s.id === props.modelValue)
          if (found) {
            selectedSubject.value = found
            console.log('展开时恢复选中科目:', found.name)
          }
        }
      })
      .catch((error) => {
        console.error('获取科目失败：', error)
      })
  }
})

onMounted(() => {
  getSubjects()
    .then((data) => {
      subjects.value = data
      // 初始化时根据modelValue设置选中状态
      if (props.modelValue) {
        const found = data.find((s) => s.id === props.modelValue)
        if (found) {
          selectedSubject.value = found
          console.log('初始化时设置选中科目:', found.name)
        }
      }
    })
    .catch((error) => {
      console.error('获取科目失败：', error)
    })
})
</script>

<template>
  <div class="subject-selector">
    <!-- 选择器触发按钮 -->
    <div
      class="selector-trigger"
      @click="!props.disabled && (isExpanded = !isExpanded)"
      :class="{ expanded: isExpanded, disabled: props.disabled }"
    >
      <span class="selected-text">
        {{ selectedSubject ? selectedSubject.name : '请选择科目' }}
      </span>
      <Icon
        name="chevron-down"
        :size="16"
        class="arrow-icon"
        :class="{ rotated: isExpanded }"
      />
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
          v-if="!showAddSubject"
          class="subject-option-add"
          @click="showAddSubject = true"
        >
          <Icon name="circle-plus" :size="16" :stroke-width="2.5" />
          <span>添加新科目</span>
        </div>
        <div class="add-form-container" v-if="showAddSubject">
          <div class="add-form-header">
            <Icon name="circle-plus" :size="16" />
            <span>添加新科目</span>
          </div>
          <input
            v-model="newSubject.name"
            type="text"
            placeholder="请输入科目名称"
            class="add-form-input"
          />
          <div class="add-form-footer">
            <div
              class="color-picker-wrap"
              :style="{ backgroundColor: newSubject.color }"
            >
              <input v-model="newSubject.color" type="color" title="选择颜色" />
            </div>
            <div class="btn-group">
              <button @click="handleAddSubject" class="btn-confirm">
                添加
              </button>
              <button @click="showAddSubject = false" class="btn-cancel">
                取消
              </button>
            </div>
          </div>
        </div>
        <div v-if="subjects.length === 0" class="no-options">暂无科目数据</div>
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

.selector-trigger.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background-color: var(--bg-secondary);
}

.selector-trigger.disabled:hover {
  border-color: var(--border-color);
  box-shadow: none;
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
  max-height: 280px;
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

/* 添加选项的单独样式 */
.subject-option-add {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 12px 15px;
  cursor: pointer;
  transition: all var(--transition-base);
  border-radius: calc(var(--radius-sm) - 2px);
  margin: 4px;
  border: 1px dashed var(--border-color);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
}

.subject-option-add:hover {
  background-color: var(--primary-light);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.subject-option-add svg {
  flex-shrink: 0;
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
  max-height: 280px;
}

.slide-down-leave-active {
  transition: all var(--transition-base);
  max-height: 280px;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
  max-height: 0;
}

/* ===== 添加表单 ===== */
.add-form-container {
  padding: 14px 16px 16px;
  border-top: 1px solid var(--divider-color);
  background: var(--bg-secondary);
  border-radius: 0 0 var(--radius-md) var(--radius-md);
}

.add-form-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.add-form-header svg {
  color: var(--primary-color);
  flex-shrink: 0;
}

.add-form-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: var(--font-size-base);
  background: var(--input-bg);
  color: var(--text-primary);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
  outline: none;
  box-sizing: border-box;
}

.add-form-input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}

.add-form-footer {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
}

/* 颜色选择器美化 */
.color-picker-wrap {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  overflow: hidden;
  border: 2px solid var(--border-color);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    border-color var(--transition-fast),
    transform var(--transition-fast);
}

.color-picker-wrap:hover {
  border-color: var(--primary-color);
  transform: scale(1.05);
}

.color-picker-wrap input[type='color'] {
  position: absolute;
  top: -6px;
  left: -6px;
  width: calc(100% + 12px);
  height: calc(100% + 12px);
  cursor: pointer;
  border: none;
  padding: 0;
  opacity: 0;
}

/* 按钮组 */
.btn-group {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.btn-confirm {
  padding: 8px 18px;
  border: none;
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  cursor: pointer;
  transition: all var(--transition-fast);
  background: var(--primary-color);
  color: white;
  letter-spacing: 0.3px;
}

.btn-confirm:hover {
  background: var(--primary-dark);
  box-shadow: var(--shadow-sm);
}

.btn-confirm:active {
  transform: scale(0.97);
}

.btn-cancel {
  padding: 8px 18px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: all var(--transition-fast);
  background: transparent;
  color: var(--text-secondary);
}

.btn-cancel:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border-color: var(--gray-400);
}
</style>
