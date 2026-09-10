<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Source } from '../types/source'
import {
  selectSourceValues,
  type SourceSelection
} from '../services/sourceSelection'
import { showWarning } from '../utils/notification'

const props = defineProps<{
  modelValue: SourceSelection
  sources: Source[]
  subjectId?: string
  disable?: boolean
}>()
const emit = defineEmits<{
  (e: 'update:modelValue', selection: SourceSelection): void
}>()

const isExpanded = ref(false)
const dropdownPosition = ref<'auto' | 'up' | 'down'>('auto')
const selectedBook = ref('')
const selectedChapter = ref('')
const selectedKnowledge = ref('')
const showAddBookInput = ref(false)
const showAddChapterInput = ref(false)
const showAddKnowledgeInput = ref(false)
const newBook = ref('')
const newChapter = ref('')
const newKnowledge = ref('')
const addedBooks = ref<string[]>([])
const addedChapters = ref<Array<{ book: string; chapter: string }>>([])
const addedKnowledges = ref<
  Array<{ book: string; chapter: string; knowledge: string }>
>([])

const uniqueSorted = (values: string[]) => [...new Set(values)].sort()
const subjectSources = computed(() =>
  props.sources.filter((source) => source.subjectId === props.subjectId)
)
const books = computed(() =>
  uniqueSorted([
    ...subjectSources.value.flatMap((source) => (source.book ? [source.book] : [])),
    ...addedBooks.value
  ])
)
const chapters = computed(() =>
  uniqueSorted([
    ...subjectSources.value.flatMap((source) =>
      source.book === selectedBook.value && source.chapter ? [source.chapter] : []
    ),
    ...addedChapters.value.flatMap((item) =>
      item.book === selectedBook.value ? [item.chapter] : []
    )
  ])
)
const knowledges = computed(() =>
  uniqueSorted([
    ...subjectSources.value.flatMap((source) =>
      source.book === selectedBook.value &&
      source.chapter === selectedChapter.value &&
      source.knowledge
        ? [source.knowledge]
        : []
    ),
    ...addedKnowledges.value.flatMap((item) =>
      item.book === selectedBook.value && item.chapter === selectedChapter.value
        ? [item.knowledge]
        : []
    )
  ])
)

const publishSelection = () => {
  emit(
    'update:modelValue',
    selectSourceValues(
      props.sources,
      props.subjectId ?? '',
      selectedBook.value || null,
      selectedChapter.value || null,
      selectedKnowledge.value || null,
      props.modelValue.kind === 'existing' ? props.modelValue.sourceId : undefined
    )
  )
}
const selectBook = (book: string) => {
  selectedBook.value = book
  selectedChapter.value = ''
  selectedKnowledge.value = ''
  showAddChapterInput.value = false
  showAddKnowledgeInput.value = false
  publishSelection()
}
const selectChapter = (chapter: string) => {
  selectedChapter.value = chapter
  selectedKnowledge.value = ''
  showAddKnowledgeInput.value = false
  publishSelection()
}
const selectKnowledge = (knowledge: string) => {
  selectedKnowledge.value = knowledge
  publishSelection()
}
const handleAddBook = () => {
  const book = newBook.value.trim()
  if (!book) return
  addedBooks.value.push(book)
  newBook.value = ''
  showAddBookInput.value = false
  selectBook(book)
}
const handleAddChapter = () => {
  const chapter = newChapter.value.trim()
  if (!chapter) return
  addedChapters.value.push({ book: selectedBook.value, chapter })
  newChapter.value = ''
  showAddChapterInput.value = false
  selectChapter(chapter)
}
const handleAddKnowledge = () => {
  const knowledge = newKnowledge.value.trim()
  if (!knowledge) return
  addedKnowledges.value.push({
    book: selectedBook.value,
    chapter: selectedChapter.value,
    knowledge
  })
  newKnowledge.value = ''
  showAddKnowledgeInput.value = false
  selectKnowledge(knowledge)
}

const selectedText = computed(() => {
  if (!selectedBook.value && !selectedChapter.value && !selectedKnowledge.value) {
    return '请选择来源'
  }
  return `${selectedBook.value} ${selectedChapter.value ? '>' : ''} ${selectedChapter.value} ${selectedKnowledge.value ? '>' : ''} ${selectedKnowledge.value}`.trim()
})

watch(
  () => [props.subjectId, props.modelValue, props.sources] as const,
  () => {
    const sourceId =
      props.modelValue.kind === 'existing' ? props.modelValue.sourceId : undefined
    const source =
      sourceId !== undefined
        ? props.sources.find((item) => item.id === sourceId)
        : undefined
    const values = props.modelValue.kind === 'new' ? props.modelValue : source
    selectedBook.value = values?.book ?? ''
    selectedChapter.value = values?.chapter ?? ''
    selectedKnowledge.value = values?.knowledge ?? ''
  },
  { immediate: true, deep: true }
)
watch(
  () => props.subjectId,
  () => {
    addedBooks.value = []
    addedChapters.value = []
    addedKnowledges.value = []
  }
)
watch(
  () => props.disable,
  (disable) => {
    if (disable) isExpanded.value = false
  }
)

const handleTriggerClick = (event: MouseEvent | TouchEvent) => {
  if (props.disable) {
    event.preventDefault()
    return
  }
  if (!props.subjectId) {
    showWarning('请选择科目', '请选择科目后再选择来源。')
    return
  }

  isExpanded.value = !isExpanded.value
  if (isExpanded.value) {
    const trigger = (event.target as HTMLElement).closest(
      '.selector-trigger'
    ) as HTMLElement | null
    if (trigger) {
      dropdownPosition.value =
        trigger.getBoundingClientRect().top < window.innerHeight / 2 ? 'down' : 'up'
    }
  }
}
</script>

<template>
  <div class="source-selector">
    <!-- 选择器触发按钮 -->
    <div
      class="selector-trigger"
      @click="handleTriggerClick"
      :class="{ expanded: isExpanded }"
    >
      <span class="selected-text">
        {{ selectedText }}
      </span>
      <Icon
        name="chevron-down"
        :size="16"
        class="arrow-icon"
        :class="{ rotated: isExpanded }"
      />
    </div>

    <!-- 多列下拉面板 -->
    <transition name="slide-down">
      <div v-if="isExpanded" :class="['dropdown-container', dropdownPosition]">
        <div class="columns-container">
          <!-- 第一列：书名 -->
          <div class="column">
            <div class="column-header">
              <span class="column-title">书名</span>
              <button
                @click="showAddBookInput = !showAddBookInput"
                class="add-icon-btn"
              >
                +
              </button>
            </div>
            <div class="column-options">
              <div
                class="column-option"
                :class="{ selected: selectedBook === book }"
                v-for="book in books"
                :key="book"
                @click="selectBook(book)"
              >
                {{ book }}
              </div>
            </div>
            <div class="column-add" v-if="showAddBookInput">
              <input
                v-model="newBook"
                type="text"
                placeholder="新书名"
                @keyup.enter="handleAddBook"
              />
              <button @click="handleAddBook" class="add-btn">确定</button>
            </div>
            <div v-if="books.length === 0" class="empty-hint">暂无数据</div>
          </div>

          <!-- 第二列：章节 -->
          <div class="column" v-if="selectedBook">
            <div class="column-header">
              <span class="column-title">章节</span>
              <button
                @click="showAddChapterInput = !showAddChapterInput"
                class="add-icon-btn"
              >
                +
              </button>
            </div>
            <div class="column-options">
              <div
                class="column-option"
                :class="{ selected: selectedChapter === chapter }"
                v-for="chapter in chapters"
                :key="chapter"
                @click="selectChapter(chapter)"
              >
                {{ chapter }}
              </div>
            </div>
            <div class="column-add" v-if="showAddChapterInput">
              <input
                v-model="newChapter"
                type="text"
                placeholder="新章节"
                @keyup.enter="handleAddChapter"
              />
              <button @click="handleAddChapter" class="add-btn">确定</button>
            </div>
            <div v-if="chapters.length === 0" class="empty-hint">暂无数据</div>
          </div>

          <!-- 第三列：知识点 -->
          <div class="column" v-if="selectedChapter">
            <div class="column-header">
              <span class="column-title">知识点</span>
              <button
                @click="showAddKnowledgeInput = !showAddKnowledgeInput"
                class="add-icon-btn"
              >
                +
              </button>
            </div>
            <div class="column-options">
              <div
                class="column-option"
                :class="{ selected: selectedKnowledge === knowledge }"
                v-for="knowledge in knowledges"
                :key="knowledge"
                @click="selectKnowledge(knowledge)"
              >
                {{ knowledge }}
              </div>
            </div>
            <div v-if="knowledges.length === 0" class="empty-hint">
              暂无数据
            </div>
            <div class="column-add" v-if="showAddKnowledgeInput">
              <input
                v-model="newKnowledge"
                type="text"
                placeholder="新知识点"
                @keyup.enter="handleAddKnowledge"
              />
              <button @click="handleAddKnowledge" class="add-btn">确定</button>
            </div>
          </div>
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
  left: 0;
  right: 0;
  z-index: var(--z-dropdown);
  background-color: var(--card-bg);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  /* 默认向下 */
  top: 100%;
  margin-top: 5px;
}

/* 向上弹出 */
.dropdown-container.up {
  top: auto;
  bottom: 100%;
  margin-top: 0;
  margin-bottom: 5px;
}

.columns-container {
  display: flex;
  min-height: 300px;
  max-height: 400px;
}

.column {
  flex: 1;
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.column:last-child {
  border-right: none;
}

.column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px;
  background-color: var(--primary-color);
  border-bottom: 1px solid var(--border-color);
}

.column-title {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.add-icon-btn {
  width: 24px;
  height: 24px;
  border: none;
  background-color: var(--primary-color);
  color: var(--white);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 18px;
  font-weight: var(--font-weight-bold);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color var(--transition-base);
}

.add-icon-btn:hover {
  background-color: var(--primary-dark);
}

.column-options {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.column-option {
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-base);
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.column-option:hover {
  background-color: var(--gray-100);
}

.column-option.selected {
  background-color: var(--primary-color);
  color: var(--white);
}

.column-add {
  padding: 10px;
  border-top: 1px solid var(--border-color);
  gap: 8px;
}

.column-add input {
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  background-color: var(--input-bg);
  color: var(--text-primary);
  width: 80%;
}

.column-add input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.add-btn {
  padding: 8px 16px;
  background-color: var(--primary-color);
  color: var(--white);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  transition: background-color var(--transition-base);
  white-space: nowrap;
}

.add-btn:hover {
  background-color: var(--primary-dark);
}

.empty-hint {
  padding: 20px;
  text-align: center;
  color: var(--gray-500);
  font-style: italic;
  font-size: var(--font-size-sm);
}

/* 向下弹出动画（默认） */
.slide-down-enter-active {
  transition: all var(--transition-base);
}

.slide-down-leave-active {
  transition: all var(--transition-base);
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* 向上弹出的动画 */
.dropdown-container.up.slide-down-enter-from,
.dropdown-container.up.slide-down-leave-to {
  transform: translateY(10px);
}

@media (max-width: 300px) {
  .columns-container {
    display: block;
  }
}
</style>
