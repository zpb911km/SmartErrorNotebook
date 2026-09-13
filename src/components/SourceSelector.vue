<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import {
  selectSourceValues,
  type SourceSelection
} from '../services/sourceSelection'
import type { Source } from '../types/source'

const props = defineProps<{
  modelValue: SourceSelection
  sources: Source[]
  subjectId?: string
  disable?: boolean
}>()
const emit = defineEmits<{
  (e: 'update:modelValue', selection: SourceSelection): void
}>()

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
    ...subjectSources.value.flatMap((source) =>
      source.book ? [source.book] : []
    ),
    ...addedBooks.value
  ])
)
const chapters = computed(() =>
  uniqueSorted([
    ...subjectSources.value.flatMap((source) =>
      source.book === selectedBook.value && source.chapter
        ? [source.chapter]
        : []
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
      props.modelValue.kind === 'existing'
        ? props.modelValue.sourceId
        : undefined
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

watch(
  () => [props.subjectId, props.modelValue, props.sources] as const,
  () => {
    const sourceId =
      props.modelValue.kind === 'existing'
        ? props.modelValue.sourceId
        : undefined
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
</script>
<template>
  <div class="source-fields">
    <q-select
      outlined
      dense
      clearable
      :model-value="selectedBook || null"
      :options="books"
      :disable="props.disable || !props.subjectId"
      label="书名"
      @update:model-value="selectBook($event || '')"
    >
      <template #after>
        <q-btn
          flat
          round
          icon="add"
          aria-label="添加书名"
          :disable="props.disable || !props.subjectId"
          @click="showAddBookInput = true"
        />
      </template>
    </q-select>
    <q-select
      outlined
      dense
      clearable
      :model-value="selectedChapter || null"
      :options="chapters"
      :disable="props.disable || !selectedBook"
      label="章节"
      @update:model-value="selectChapter($event || '')"
    >
      <template #after>
        <q-btn
          flat
          round
          icon="add"
          aria-label="添加章节"
          :disable="props.disable || !selectedBook"
          @click="showAddChapterInput = true"
        />
      </template>
    </q-select>
    <q-select
      outlined
      dense
      clearable
      :model-value="selectedKnowledge || null"
      :options="knowledges"
      :disable="props.disable || !selectedChapter"
      label="知识点"
      @update:model-value="selectKnowledge($event || '')"
    >
      <template #after>
        <q-btn
          flat
          round
          icon="add"
          aria-label="添加知识点"
          :disable="props.disable || !selectedChapter"
          @click="showAddKnowledgeInput = true"
        />
      </template>
    </q-select>
    <div v-if="!props.subjectId" class="text-caption text-muted">
      选择科目后可设置来源
    </div>
    <q-dialog v-model="showAddBookInput">
      <q-card class="dialog-card">
        <q-card-section class="text-h6"> 添加书名 </q-card-section
        ><q-card-section>
          <q-input
            v-model="newBook"
            outlined
            autofocus
            label="书名"
            @keyup.enter="handleAddBook"
          /> </q-card-section
        ><q-card-actions align="right">
          <q-btn v-close-popup flat label="取消" /><q-btn
            color="primary"
            label="添加"
            :disable="!newBook.trim()"
            @click="handleAddBook"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
    <q-dialog v-model="showAddChapterInput">
      <q-card class="dialog-card">
        <q-card-section class="text-h6"> 添加章节 </q-card-section
        ><q-card-section>
          <q-input
            v-model="newChapter"
            outlined
            autofocus
            label="章节"
            @keyup.enter="handleAddChapter"
          /> </q-card-section
        ><q-card-actions align="right">
          <q-btn v-close-popup flat label="取消" /><q-btn
            color="primary"
            label="添加"
            :disable="!newChapter.trim()"
            @click="handleAddChapter"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
    <q-dialog v-model="showAddKnowledgeInput">
      <q-card class="dialog-card">
        <q-card-section class="text-h6"> 添加知识点 </q-card-section
        ><q-card-section>
          <q-input
            v-model="newKnowledge"
            outlined
            autofocus
            label="知识点"
            @keyup.enter="handleAddKnowledge"
          /> </q-card-section
        ><q-card-actions align="right">
          <q-btn v-close-popup flat label="取消" /><q-btn
            color="primary"
            label="添加"
            :disable="!newKnowledge.trim()"
            @click="handleAddKnowledge"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>
