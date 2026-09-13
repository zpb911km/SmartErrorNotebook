<script setup lang="ts">
import { computed } from 'vue'

import { createSourceCatalog } from '../services/sourceCatalog'
import type { Source, Subject } from '../types'
import ResponsiveFilterPanel from './ResponsiveFilterPanel.vue'

interface Filters {
  subjectId: string
  book: string
  chapter: string
  knowledge: string
  keyword?: string
  tags?: string[]
  date_range?: string
}
type Sort = 'none' | 'asc' | 'desc'
const props = defineProps<{
  modelValue: Filters
  sources: Source[]
  subjects: Subject[]
  tags?: string[]
  difficultySort?: Sort
  masterySort?: Sort
}>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: Filters): void
  (e: 'update:difficultySort', value: Sort): void
  (e: 'update:masterySort', value: Sort): void
}>()
const catalog = createSourceCatalog(() => props.sources)
const books = computed(() =>
  props.modelValue.subjectId ? catalog.getBooks(props.modelValue.subjectId) : []
)
const chapters = computed(() =>
  catalog.getChapters(props.modelValue.book, props.modelValue.subjectId)
)
const knowledges = computed(() =>
  catalog.getKnowledges(
    props.modelValue.book,
    props.modelValue.chapter,
    props.modelValue.subjectId
  )
)
const sorts = [
  { label: '默认顺序', value: 'none' },
  { label: '由低到高', value: 'asc' },
  { label: '由高到低', value: 'desc' }
]
const dates = [
  { label: '全部时间', value: 'all' },
  { label: '最近 7 天', value: '7days' },
  { label: '最近 30 天', value: '30days' },
  { label: '最近 90 天', value: '90days' }
]
function update(key: keyof Filters, value: string | string[] | null) {
  const next = { ...props.modelValue, [key]: value ?? '' }
  if (key === 'subjectId') {
    next.book = ''
    next.chapter = ''
    next.knowledge = ''
  }
  if (key === 'book') {
    next.chapter = ''
    next.knowledge = ''
  }
  if (key === 'chapter') next.knowledge = ''
  emit('update:modelValue', next)
}
</script>
<template>
  <ResponsiveFilterPanel>
    <q-select
      outlined
      dense
      clearable
      :model-value="modelValue.subjectId || null"
      :options="subjects"
      option-label="name"
      option-value="id"
      emit-value
      map-options
      label="科目"
      @update:model-value="update('subjectId', $event)"
    />
    <q-select
      outlined
      dense
      clearable
      :model-value="modelValue.book || null"
      :options="books"
      :disable="!modelValue.subjectId"
      label="书名"
      @update:model-value="update('book', $event)"
    />
    <q-select
      outlined
      dense
      clearable
      :model-value="modelValue.chapter || null"
      :options="chapters"
      :disable="!modelValue.book"
      label="章节"
      @update:model-value="update('chapter', $event)"
    />
    <q-select
      outlined
      dense
      clearable
      :model-value="modelValue.knowledge || null"
      :options="knowledges"
      :disable="!modelValue.chapter"
      label="知识点"
      @update:model-value="update('knowledge', $event)"
    />
    <q-select
      v-if="tags"
      outlined
      dense
      multiple
      use-chips
      :model-value="modelValue.tags"
      :options="tags"
      label="错因标签"
      @update:model-value="update('tags', $event)"
    />
    <q-select
      v-if="modelValue.date_range !== undefined"
      outlined
      dense
      :model-value="modelValue.date_range"
      :options="dates"
      emit-value
      map-options
      label="时间范围"
      @update:model-value="update('date_range', $event)"
    />
    <q-select
      v-if="difficultySort !== undefined"
      outlined
      dense
      :model-value="difficultySort"
      :options="sorts"
      emit-value
      map-options
      label="难度排序"
      @update:model-value="emit('update:difficultySort', $event)"
    />
    <q-select
      v-if="masterySort !== undefined"
      outlined
      dense
      :model-value="masterySort"
      :options="sorts"
      emit-value
      map-options
      label="掌握程度排序"
      @update:model-value="emit('update:masterySort', $event)"
    />
  </ResponsiveFilterPanel>
</template>
