<script lang="ts">
import type { Source, Subject, Tag } from '@/types'

export interface LibraryFilterPanelData {
  subjects: Subject[]
  sources: Source[]
  tags?: Tag[]
}

export interface LibraryFilterPanelModelValue {
  filter: LibraryFilter
  sort: LibrarySort
}

export interface LibraryFilter {
  subjectId: string
  book: string
  chapter: string
  knowledge: string
  keyword?: string
  tagIds?: string[]
  dateRange?: 'all' | '7days' | '30days' | '90days'
}

export interface LibrarySort {
  difficulty?: LibrarySortDirection
  mastery?: LibrarySortDirection
}

export type LibrarySortDirection = 'none' | 'asc' | 'desc'

export function selectLibrarySort(
  sort: LibrarySort,
  key: keyof LibrarySort,
  value: LibrarySortDirection
): LibrarySort {
  const next = { ...sort, [key]: value }
  const other = key === 'difficulty' ? 'mastery' : 'difficulty'
  if (value !== 'none' && next[other] !== undefined) next[other] = 'none'
  return next
}

export interface LibraryActiveFilterItem {
  key: string
  label: string
}

export function createEmptyLibraryFilterPanelModelValue(
  overrides: {
    filter?: Partial<LibraryFilter>
    sort?: LibrarySort
  } = {}
): LibraryFilterPanelModelValue {
  return {
    filter: {
      subjectId: '',
      book: '',
      chapter: '',
      knowledge: '',
      ...overrides.filter
    },
    sort: { ...overrides.sort }
  }
}
</script>

<script setup lang="ts">
import { computed } from 'vue'

import { createSourceCatalog } from '../services/sourceCatalog'
import ResponsiveFilterPanel from './ResponsiveFilterPanel.vue'

const props = defineProps<{
  data: LibraryFilterPanelData
  modelValue: LibraryFilterPanelModelValue
}>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: LibraryFilterPanelModelValue): void
}>()

const catalog = createSourceCatalog(() => props.data.sources)
const books = computed(() =>
  props.modelValue.filter.subjectId
    ? catalog.getBooks(props.modelValue.filter.subjectId)
    : []
)
const chapters = computed(() =>
  catalog.getChapters(
    props.modelValue.filter.book,
    props.modelValue.filter.subjectId
  )
)
const knowledges = computed(() =>
  catalog.getKnowledges(
    props.modelValue.filter.book,
    props.modelValue.filter.chapter,
    props.modelValue.filter.subjectId
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

const activeFilterItems = computed(() => {
  const { filter, sort } = props.modelValue
  const values: LibraryActiveFilterItem[] = []
  if (filter.subjectId) {
    const subject = props.data.subjects.find(
      (item) => item.id === filter.subjectId
    )
    if (subject) values.push({ key: 'subjectId', label: subject.name })
  }
  if (filter.dateRange && filter.dateRange !== 'all') {
    const labels = {
      '7days': '最近7天',
      '30days': '最近30天',
      '90days': '最近90天'
    }
    values.push({ key: 'dateRange', label: labels[filter.dateRange] })
  }
  if (filter.tagIds?.length) {
    const names = filter.tagIds.flatMap((id) => {
      const tag = props.data.tags?.find((item) => item.id === id)
      return tag ? [tag.name] : []
    })
    if (names.length) {
      values.push({
        key: 'tagIds',
        label:
          names.length === 1 ? names[0] : `${names[0]} +${names.length - 1}`
      })
    }
  }
  if (sort.difficulty && sort.difficulty !== 'none') {
    values.push({
      key: 'difficulty',
      label: sort.difficulty === 'asc' ? '难度正序排序' : '难度倒序排序'
    })
  }
  if (sort.mastery && sort.mastery !== 'none') {
    values.push({
      key: 'mastery',
      label: sort.mastery === 'asc' ? '掌握程度正序排序' : '掌握程度倒序排序'
    })
  }
  if (filter.book) values.push({ key: 'book', label: filter.book })
  if (filter.chapter) values.push({ key: 'chapter', label: filter.chapter })
  if (filter.knowledge) {
    values.push({ key: 'knowledge', label: filter.knowledge })
  }
  return values
})

function updateFilter(
  key: keyof LibraryFilter,
  value: string | string[] | null
) {
  const filter = { ...props.modelValue.filter, [key]: value ?? '' }
  if (key === 'subjectId') {
    filter.book = ''
    filter.chapter = ''
    filter.knowledge = ''
  } else if (key === 'book') {
    filter.chapter = ''
    filter.knowledge = ''
  } else if (key === 'chapter') {
    filter.knowledge = ''
  }
  emit('update:modelValue', { ...props.modelValue, filter })
}

function updateSort(key: keyof LibrarySort, value: LibrarySortDirection) {
  emit('update:modelValue', {
    ...props.modelValue,
    sort: selectLibrarySort(props.modelValue.sort, key, value)
  })
}

function clearFilterItem(key: string) {
  const filter = { ...props.modelValue.filter }
  const sort = { ...props.modelValue.sort }
  if (key === 'subjectId') {
    filter.subjectId = ''
    filter.book = ''
    filter.chapter = ''
    filter.knowledge = ''
  } else if (key === 'book') {
    filter.book = ''
    filter.chapter = ''
    filter.knowledge = ''
  } else if (key === 'chapter') {
    filter.chapter = ''
    filter.knowledge = ''
  } else if (key === 'knowledge') {
    filter.knowledge = ''
  } else if (key === 'dateRange') {
    filter.dateRange = 'all'
  } else if (key === 'tagIds') {
    filter.tagIds = []
  } else if (key === 'difficulty') {
    sort.difficulty = 'none'
  } else if (key === 'mastery') {
    sort.mastery = 'none'
  }
  emit('update:modelValue', { ...props.modelValue, filter, sort })
}

function clearAllFilterItems() {
  const current = props.modelValue.filter
  emit(
    'update:modelValue',
    createEmptyLibraryFilterPanelModelValue({
      filter: {
        ...(current.keyword !== undefined ? { keyword: '' } : {}),
        ...(current.tagIds !== undefined ? { tagIds: [] } : {}),
        ...(current.dateRange !== undefined
          ? { dateRange: 'all' as const }
          : {})
      },
      sort: {
        ...(props.modelValue.sort.difficulty !== undefined
          ? { difficulty: 'none' as const }
          : {}),
        ...(props.modelValue.sort.mastery !== undefined
          ? { mastery: 'none' as const }
          : {})
      }
    })
  )
}

defineExpose({ activeFilterItems, clearFilterItem, clearAllFilterItems })
</script>

<template>
  <div>
    <ResponsiveFilterPanel>
      <q-select
        outlined
        dense
        clearable
        :model-value="modelValue.filter.subjectId || null"
        :options="data.subjects"
        option-label="name"
        option-value="id"
        emit-value
        map-options
        label="科目"
        @update:model-value="updateFilter('subjectId', $event)"
      />
      <q-select
        outlined
        dense
        clearable
        :model-value="modelValue.filter.book || null"
        :options="books"
        :disable="!modelValue.filter.subjectId"
        label="书名"
        @update:model-value="updateFilter('book', $event)"
      />
      <q-select
        outlined
        dense
        clearable
        :model-value="modelValue.filter.chapter || null"
        :options="chapters"
        :disable="!modelValue.filter.book"
        label="章节"
        @update:model-value="updateFilter('chapter', $event)"
      />
      <q-select
        outlined
        dense
        clearable
        :model-value="modelValue.filter.knowledge || null"
        :options="knowledges"
        :disable="!modelValue.filter.chapter"
        label="知识点"
        @update:model-value="updateFilter('knowledge', $event)"
      />
      <q-select
        v-if="data.tags"
        outlined
        dense
        multiple
        use-chips
        :model-value="modelValue.filter.tagIds"
        :options="data.tags"
        option-label="name"
        option-value="id"
        emit-value
        map-options
        label="错因标签"
        @update:model-value="updateFilter('tagIds', $event)"
      />
      <q-select
        v-if="modelValue.filter.dateRange !== undefined"
        outlined
        dense
        :model-value="modelValue.filter.dateRange"
        :options="dates"
        emit-value
        map-options
        label="时间范围"
        @update:model-value="updateFilter('dateRange', $event)"
      />
      <q-select
        v-if="modelValue.sort.difficulty !== undefined"
        outlined
        dense
        :model-value="modelValue.sort.difficulty"
        :options="sorts"
        emit-value
        map-options
        label="难度排序"
        @update:model-value="updateSort('difficulty', $event)"
      />
      <q-select
        v-if="modelValue.sort.mastery !== undefined"
        outlined
        dense
        :model-value="modelValue.sort.mastery"
        :options="sorts"
        emit-value
        map-options
        label="掌握程度排序"
        @update:model-value="updateSort('mastery', $event)"
      />
    </ResponsiveFilterPanel>
  </div>
</template>
