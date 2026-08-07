<script setup lang="ts">
import { onMounted, ref, watch, computed } from 'vue'
import { getErrorTags } from '../apis/errorTags'
import { ErrorTags } from '../types'
import { showInfo } from '../utils/notification'

// 定义选中的标签信息类型
interface SelectedTagInfo {
  name: string
  color: string
}

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

const errorTags = ref<ErrorTags[]>([])
const selectedTags = ref<SelectedTagInfo[]>([])
const isExpanded = ref(false)

// 过滤掉已删除的标签
const filteredErrorTags = computed(() => {
  return errorTags.value.filter((tag) => !tag.name.startsWith('[已删除]'))
})
const showAddErrorTag = ref(false)
const newErrorTag = ref<Omit<ErrorTags, 'id' | 'question_id'>>({
  name: '',
  color: getRandomRGBColor()
})

const props = defineProps<{
  currentTags?: SelectedTagInfo[]
}>()

const emit = defineEmits<{
  (e: 'select', tags: SelectedTagInfo[]): void
}>()

// 显示文本
const displayText = computed(() => {
  if (selectedTags.value.length === 0) {
    return '请选择错因'
  } else if (selectedTags.value.length === 1) {
    return selectedTags.value[0].name
  } else {
    return `已选择 ${selectedTags.value.length} 个错因`
  }
})

watch(
  () => props.currentTags,
  (newVal) => {
    if (newVal) {
      selectedTags.value = [...newVal]
    } else {
      selectedTags.value = []
    }
  },
  { immediate: true }
)

const handleClick = (errorTag: ErrorTags, event: Event) => {
  event.stopPropagation()
  const index = selectedTags.value.findIndex(
    (tag) => tag.name === errorTag.name && tag.color === errorTag.color
  )
  if (index > -1) {
    // 取消选择
    selectedTags.value.splice(index, 1)
  } else {
    // 添加选择（只保存name和color）
    selectedTags.value.push({
      name: errorTag.name,
      color: errorTag.color
    })
  }
  emit('select', [...selectedTags.value])
}

const handleAddErrorTag = () => {
  // 不再调用API创建标签，只是添加到已选列表
  console.log('添加错因标签', newErrorTag.value)
  showInfo('添加成功', '错因标签添加成功')

  // 添加到已选列表
  selectedTags.value.push({
    name: newErrorTag.value.name,
    color: newErrorTag.value.color
  })
  errorTags.value.push({
    id: '',
    question_id: '',
    name: newErrorTag.value.name,
    color: newErrorTag.value.color
  })

  emit('select', [...selectedTags.value])
  showAddErrorTag.value = false

  // 重置新标签表单
  newErrorTag.value = {
    name: '',
    color: getRandomRGBColor()
  }
}

// 检查标签是否被选中
const isTagSelected = (errorTag: ErrorTags) => {
  return selectedTags.value.some(
    (tag) => tag.name === errorTag.name && tag.color === errorTag.color
  )
}

// 移除标签
const removeTag = (index: number) => {
  selectedTags.value.splice(index, 1)
  emit('select', [...selectedTags.value])
}

watch(isExpanded, (newVal) => {
  if (newVal) {
    getErrorTags()
      .then((data) => {
        errorTags.value = data
      })
      .catch((error) => {
        console.error('获取错因标签失败：', error)
      })
  }
})

onMounted(() => {
  getErrorTags()
    .then((data) => {
      errorTags.value = data
    })
    .catch((error) => {
      console.error('获取错因标签失败：', error)
    })
})
</script>

<template>
  <div class="error-tag-selector">
    <!-- 已选标签展示 -->
    <div class="selected-tags" v-if="selectedTags.length > 0">
      <div
        class="tag-chip"
        v-for="(tag, index) in selectedTags"
        :key="`${tag.name}-${tag.color}`"
        :style="{ backgroundColor: tag.color + '20', color: tag.color }"
      >
        {{ tag.name }}
        <span class="remove-tag" @click.stop="removeTag(index)"
          ><Icon name="x" :size="14"
        /></span>
      </div>
    </div>

    <!-- 选择器触发按钮 -->
    <div
      class="selector-trigger"
      @click="isExpanded = !isExpanded"
      :class="{ expanded: isExpanded }"
    >
      <span class="selected-text">
        {{ displayText }}
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
          class="error-tag-option"
          v-for="errorTag in filteredErrorTags"
          :key="errorTag.id"
          @click="handleClick(errorTag, $event)"
          :class="{ selected: isTagSelected(errorTag) }"
        >
          <div class="option-left">
            <span
              class="checkbox"
              :class="{ checked: isTagSelected(errorTag) }"
            ></span>
            <span class="option-name">{{ errorTag.name }}</span>
          </div>
          <div
            class="color-indicator"
            :style="{ backgroundColor: errorTag.color }"
          ></div>
        </div>
        <div
          class="error-tag-option-add"
          v-if="!showAddErrorTag"
          @click="showAddErrorTag = true"
        >
          <Icon name="circle-plus" :size="16" :stroke-width="2.5" />
          <span>添加新错因</span>
        </div>
        <div class="add-form-container" v-if="showAddErrorTag">
          <div class="add-form-header">
            <Icon name="circle-plus" :size="16" />
            <span>添加新错因</span>
          </div>
          <input
            v-model="newErrorTag.name"
            type="text"
            placeholder="请输入错因名称"
            class="add-form-input"
          />
          <div class="add-form-footer">
            <div
              class="color-picker-wrap"
              :style="{ backgroundColor: newErrorTag.color }"
            >
              <input
                v-model="newErrorTag.color"
                type="color"
                title="选择颜色"
              />
            </div>
            <div class="btn-group">
              <button @click="handleAddErrorTag" class="btn-confirm">
                添加
              </button>
              <button @click="showAddErrorTag = false" class="btn-cancel">
                取消
              </button>
            </div>
          </div>
        </div>
        <div v-if="filteredErrorTags.length === 0" class="no-options">
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
  gap: 4px;
  padding: 6px 12px;
  border-radius: 16px;
  font-size: 13px;
  font-weight: 500;
}

.remove-tag {
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.2s;
  font-size: 14px;
  line-height: 1;
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
  max-height: 280px;
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

.error-tag-option-add {
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

.error-tag-option-add:hover {
  background-color: var(--primary-light);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.error-tag-option-add svg {
  flex-shrink: 0;
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

/* ===== 添加表单（与 SubjectSelector 保持一致） ===== */
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

/* ===== 暗色模式适配 ===== */
body.dark-theme .error-tag-option:hover {
  background-color: rgba(255, 255, 255, 0.08) !important;
}

body.dark-theme .error-tag-option-add {
  border-color: var(--gray-500);
}

body.dark-theme .error-tag-option-add:hover {
  background-color: rgba(25, 118, 210, 0.15);
  border-color: var(--primary-color);
}
</style>
