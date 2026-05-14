<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { getBooks, getChapters, getKnowledges, getOrCreateSourceId, getSource } from '../apis/sources';
import { Source } from '../types';
import { showError, showWarning } from '../utils/notification';

const selectedSource = ref<Source | null>(null);
const isExpanded = ref(false);
const dropdownPosition = ref<'auto' | 'up' | 'down'>('auto');

// 分层选择的数据
const books = ref<string[]>([]);
const chapters = ref<string[]>([]);
const knowledges = ref<string[]>([]);

// 当前选择的层级
const selectedBook = ref<string>('');
const selectedChapter = ref<string>('');
const selectedKnowledge = ref<string>('');

// 添加新项的输入框
const showAddBookInput = ref(false);
const showAddChapterInput = ref(false);
const showAddKnowledgeInput = ref(false);
const newBook = ref<string>('');
const newChapter = ref<string>('');
const newKnowledge = ref<string>('');

const props = defineProps<{
  currentSourceId: string;
  subjectId?: string;
  disable?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', source_id: string): void;
}>();


const atLeastOneSelected = () => {
  return `${selectedBook.value || ''} ${selectedChapter.value || ''} ${selectedKnowledge.value || ''}`.trim().length > 0;
};

const resetSelection = () => {
  selectedBook.value = '';
  selectedChapter.value = '';
  selectedKnowledge.value = '';
  chapters.value = [];
  knowledges.value = [];
  showAddBookInput.value = false;
  showAddChapterInput.value = false;
  showAddKnowledgeInput.value = false;
};

const loadBooks = () => {
  getBooks(props.subjectId)
    .then(data => {
      books.value = data;
    })
    .catch(error => {
      console.error('获取书名失败：', error);
      books.value = [];
    });
};

const handleAddBook = () => {
  if (!newBook.value.trim()) return;

  books.value.push(newBook.value);
  selectedBook.value = newBook.value;
  newBook.value = '';
  showAddBookInput.value = false;
  addAllToBackend();
};

const handleAddChapter = () => {
  if (!newChapter.value.trim()) return;

  chapters.value.push(newChapter.value);
  selectedChapter.value = newChapter.value;
  newChapter.value = '';
  showAddChapterInput.value = false;
  addAllToBackend();
};

const handleAddKnowledge = () => {
  if (!newKnowledge.value.trim()) return;

  knowledges.value.push(newKnowledge.value);
  selectedKnowledge.value = newKnowledge.value;
  newKnowledge.value = '';
  showAddKnowledgeInput.value = false;
  addAllToBackend();
};

const addAllToBackend = async () => {
  if (props.subjectId && props.subjectId !== '' && atLeastOneSelected()) {
    await getOrCreateSourceId({
      subject_id: props.subjectId,
      book: selectedBook.value,
      chapter: selectedChapter.value === ''? undefined : selectedChapter.value,
      knowledge: selectedKnowledge.value === ''? undefined : selectedKnowledge.value,
    }).catch(error => {
      showError('添加失败', '添加来源失败' + error);
    });
  }
}

const selectedText = computed(() => {
  if (atLeastOneSelected()) {
    console.log("selected:", selectedBook.value, selectedChapter.value, selectedKnowledge.value)
    return `${selectedBook.value || ''} ${selectedChapter.value? '>': ''} ${selectedChapter.value || ''} ${selectedKnowledge.value? '>': ''} ${selectedKnowledge.value || ''}`.trim();
  }
  return '请选择来源';
});

const loadSourceById = async (sourceId: string) => {
  if (!sourceId) {
    selectedSource.value = null;
    return;
  }

  try {
    const source = await getSource(sourceId);
    if (source.book) {
      selectedBook.value = source.book;
    }
    if (source.chapter) {
      selectedChapter.value = source.chapter;
    }
    if (source.knowledge) {
      selectedKnowledge.value = source.knowledge;
    }
    if (props.subjectId) {
      books.value = await getBooks(props.subjectId);
    }
    console.log("data:", selectedBook.value, selectedChapter.value, selectedKnowledge.value)
    // 预加载下一级数据
    if (source.book) {
      getChapters(source.book).then(data => {
        chapters.value = data;
        if (source.chapter && source.book) {
          getKnowledges(source.book, source.chapter).then(data => {
            knowledges.value = data;
          });
        }
      });
    }
  } catch (error) {
    console.error('获取来源失败：', error);
  }
};

// 然后定义 watch
// 当选择书名时,加载章节列表
watch(selectedBook, (newBook) => {
  if (newBook) {
    getChapters(newBook)
      .then(data => {
        chapters.value = data;
        selectedChapter.value = selectedChapter.value === undefined? '': selectedChapter.value;
        selectedKnowledge.value = selectedKnowledge.value === undefined? '': selectedKnowledge.value;
        showAddChapterInput.value = false;
        showAddKnowledgeInput.value = false;
      })
      .catch(error => {
        console.error('获取章节失败：', error);
        chapters.value = [];
      });
  }
});

// 当选择章节时,加载知识点列表
watch(selectedChapter, (newChapter) => {
  if (newChapter && selectedBook.value) {
    getKnowledges(selectedBook.value, newChapter)
      .then(data => {
        knowledges.value = data;
        selectedKnowledge.value = selectedKnowledge.value === undefined? '': selectedKnowledge.value;
        showAddKnowledgeInput.value = false;
      })
      .catch(error => {
        console.error('获取知识点失败：', error);
        knowledges.value = [];
      });
  }
});

// 当subjectId变化时,重新加载书名列表
watch(
  () => props.subjectId,
  () => {
    resetSelection();
    loadBooks();
  }, { immediate: true }
);

watch(
  () => props.currentSourceId,
  (newVal) => {
    console.log('currentSourceId changed:', newVal)
    loadSourceById(newVal);
  }, { immediate: true }
);

watch(isExpanded, async (newVal) => {
  if (!props.subjectId && newVal) {
    isExpanded.value = false;
    showWarning('请选择科目', '请选择科目后再选择来源。')
  }
  if (newVal) {
    resetSelection();
    loadBooks();
  }
  let sourceId = "";
  if (!newVal && props.subjectId && props.subjectId !== '' && atLeastOneSelected()) {
    await getOrCreateSourceId({
      subject_id: props.subjectId,
      book: selectedBook.value,
      chapter: selectedChapter.value === ''? undefined : selectedChapter.value,
      knowledge: selectedKnowledge.value === ''? undefined : selectedKnowledge.value,
    }).then((id) => {
      sourceId = id;
    }).catch(error => {
      showError('添加失败', '添加来源失败' + error);
    });
  }
  if (sourceId) {
    emit('select', sourceId);
  }
})

// 处理触发器点击：如果按钮在下半屏则向上弹出，否则向下弹出
const handleTriggerClick = async (e: MouseEvent | TouchEvent) => {
  // disable
  if (props.disable && props.disable === true) {
    e.preventDefault();
    return;
  }

  // 先切换展开状态
  isExpanded.value = !isExpanded.value;

  // 只在下半屏才设为向上弹出，默认向下
  if (isExpanded.value) {
    // 获取正确的触发器元素（可能是事件冒泡后的子元素）
    const triggerElement = (e.target as HTMLElement).closest('.selector-trigger') as HTMLElement;
    if (triggerElement) {
      const rect = triggerElement.getBoundingClientRect();
      dropdownPosition.value = rect.top < window.innerHeight / 2 ? 'down' : 'up';
    }
    resetSelection();
    loadBooks();
  } else {
    if (props.subjectId && props.subjectId !== '' && atLeastOneSelected()) {
      await getOrCreateSourceId({
        subject_id: props.subjectId,
        book: selectedBook.value,
        chapter: selectedChapter.value === ''? undefined : selectedChapter.value,
        knowledge: selectedKnowledge.value === ''? undefined : selectedKnowledge.value,
      }).then((id) => {
        emit('select', id);
      }).catch(error => {
        showError('添加失败', '添加来源失败' + error);
      });
    }
  }
};
</script>

<template>
  <div class="source-selector">
    <!-- 选择器触发按钮 -->
    <div class="selector-trigger" @click="handleTriggerClick" :class="{ 'expanded': isExpanded }">
      <span class="selected-text">
        {{ selectedText }}
      </span>
      <svg class="arrow-icon" :class="{ 'rotated': isExpanded }" xmlns="http://www.w3.org/2000/svg" width="16"
        height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
        stroke-linejoin="round">
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </div>

    <!-- 多列下拉面板 -->
    <transition name="slide-down">
      <div v-if="isExpanded" :class="['dropdown-container', dropdownPosition]">
        <div class="columns-container">
          <!-- 第一列：书名 -->
          <div class="column">
            <div class="column-header">
              <span class="column-title">书名</span>
              <button @click="showAddBookInput = !showAddBookInput" class="add-icon-btn">+</button>
            </div>
            <div class="column-options">
              <div class="column-option" :class="{ 'selected': selectedBook === book }" v-for="book in books"
                :key="book" @click="selectedBook = book">
                {{ book }}
              </div>
            </div>
            <div class="column-add" v-if="showAddBookInput">
              <input v-model="newBook" type="text" placeholder="新书名" @keyup.enter="handleAddBook">
              <button @click="handleAddBook" class="add-btn">确定</button>
            </div>
            <div v-if="books.length === 0" class="empty-hint">
              暂无数据
            </div>
          </div>

          <!-- 第二列：章节 -->
          <div class="column" v-if="selectedBook">
            <div class="column-header">
              <span class="column-title">章节</span>
              <button @click="showAddChapterInput = !showAddChapterInput" class="add-icon-btn">+</button>
            </div>
            <div class="column-options">
              <div class="column-option" :class="{ 'selected': selectedChapter === chapter }"
                v-for="chapter in chapters" :key="chapter" @click="selectedChapter = chapter">
                {{ chapter }}
              </div>
            </div>
            <div class="column-add" v-if="showAddChapterInput">
              <input v-model="newChapter" type="text" placeholder="新章节" @keyup.enter="handleAddChapter">
              <button @click="handleAddChapter" class="add-btn">确定</button>
            </div>
            <div v-if="chapters.length === 0" class="empty-hint">
              暂无数据
            </div>
          </div>

          <!-- 第三列：知识点 -->
          <div class="column" v-if="selectedChapter">
            <div class="column-header">
              <span class="column-title">知识点</span>
              <button @click="showAddKnowledgeInput = !showAddKnowledgeInput" class="add-icon-btn">+</button>
            </div>
            <div class="column-options">
              <div class="column-option" :class="{ 'selected': selectedKnowledge === knowledge }"
                v-for="knowledge in knowledges" :key="knowledge" @click="selectedKnowledge = knowledge">
                {{ knowledge }}
              </div>
            </div>
            <div v-if="knowledges.length === 0" class="empty-hint">
              暂无数据
            </div>
            <div class="column-add" v-if="showAddKnowledgeInput">
              <input v-model="newKnowledge" type="text" placeholder="新知识点" @keyup.enter="handleAddKnowledge">
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
  background-color: var(--gray-100);
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