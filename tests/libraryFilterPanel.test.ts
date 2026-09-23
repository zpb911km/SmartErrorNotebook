// @vitest-environment happy-dom
import { Quasar } from 'quasar'
import { afterEach, expect, it } from 'vitest'
import { createApp, h, nextTick, ref } from 'vue'

import AppIcon from '../src/components/AppIcon.vue'
import LibraryFilterPanel, {
  createEmptyLibraryFilterPanelModelValue,
  type LibraryFilterPanelData,
  type LibraryFilterPanelModelValue,
  selectLibrarySort
} from '../src/components/LibraryFilterPanel.vue'
import { quasarOptions } from '../src/quasar'

const cleanups: Array<() => void> = []

function mountPanel(
  data: LibraryFilterPanelData,
  initial: LibraryFilterPanelModelValue
) {
  const model = ref(initial)
  const panel = ref<InstanceType<typeof LibraryFilterPanel> | null>(null)
  const element = document.createElement('div')
  document.body.append(element)
  const app = createApp(() =>
    h(LibraryFilterPanel, {
      ref: panel,
      data,
      modelValue: model.value,
      'onUpdate:modelValue': (value: LibraryFilterPanelModelValue) => {
        model.value = value
      }
    })
  )
  app.use(Quasar, quasarOptions)
  app.component('AppIcon', AppIcon)
  app.mount(element)
  cleanups.push(() => {
    app.unmount()
    element.remove()
  })
  return { element, model, panel }
}

afterEach(() => {
  cleanups
    .splice(0)
    .reverse()
    .forEach((cleanup) => cleanup())
})

it('creates independent empty panel model values and applies overrides', () => {
  const first = createEmptyLibraryFilterPanelModelValue()
  const second = createEmptyLibraryFilterPanelModelValue({
    filter: { keyword: '', tagIds: [], dateRange: 'all' },
    sort: { difficulty: 'none' }
  })

  expect(first).toEqual({
    filter: {
      subjectId: '',
      book: '',
      chapter: '',
      knowledge: ''
    },
    sort: {}
  })
  expect(second).toEqual({
    filter: {
      subjectId: '',
      book: '',
      chapter: '',
      knowledge: '',
      keyword: '',
      tagIds: [],
      dateRange: 'all'
    },
    sort: { difficulty: 'none' }
  })
  expect(first).not.toBe(second)
  expect(first.filter).not.toBe(second.filter)
  expect(first.sort).not.toBe(second.sort)
})

it('switches between mutually exclusive sorts without enabling a cleared sort', () => {
  const initial = { difficulty: 'none', mastery: 'none' } as const
  const difficulty = selectLibrarySort(initial, 'difficulty', 'desc')
  expect(difficulty).toEqual({ difficulty: 'desc', mastery: 'none' })
  expect(initial).toEqual({ difficulty: 'none', mastery: 'none' })

  const mastery = selectLibrarySort(difficulty, 'mastery', 'asc')
  expect(mastery).toEqual({ difficulty: 'none', mastery: 'asc' })
  expect(selectLibrarySort(mastery, 'mastery', 'none')).toEqual({
    difficulty: 'none',
    mastery: 'none'
  })
  expect(selectLibrarySort(mastery, 'difficulty', 'asc')).toEqual({
    difficulty: 'asc',
    mastery: 'none'
  })
  expect(
    selectLibrarySort({ difficulty: 'none' }, 'difficulty', 'asc')
  ).toEqual({ difficulty: 'asc' })
})

it('owns active filter removal and complete reset behavior', async () => {
  const { model, panel } = mountPanel(
    {
      subjects: [
        {
          id: 'math',
          name: '数学',
          color: '#123456',
          createdAt: '2026-01-01T00:00:00Z',
          updatedAt: '2026-01-01T00:00:00Z'
        }
      ],
      sources: [],
      tags: [{ id: 'tag', name: '计算', color: '#654321' }]
    },
    {
      filter: {
        subjectId: 'math',
        book: '教材',
        chapter: '第一章',
        knowledge: '函数',
        keyword: '极限',
        tagIds: ['tag'],
        dateRange: '7days'
      },
      sort: { difficulty: 'desc', mastery: 'none' }
    }
  )

  expect(panel.value?.activeFilterItems).toEqual(
    expect.arrayContaining([
      { key: 'subjectId', label: '数学' },
      { key: 'tagIds', label: '计算' }
    ])
  )
  panel.value?.clearFilterItem('subjectId')
  await nextTick()
  expect(model.value.filter).toMatchObject({
    subjectId: '',
    book: '',
    chapter: '',
    knowledge: ''
  })

  panel.value?.clearAllFilterItems()
  await nextTick()
  expect(model.value.filter).toEqual({
    subjectId: '',
    book: '',
    chapter: '',
    knowledge: '',
    keyword: '',
    tagIds: [],
    dateRange: 'all'
  })
  expect(model.value.sort).toEqual({ difficulty: 'none', mastery: 'none' })
})

it('omits optional controls and preserves their absence when clearing', async () => {
  const { element, model, panel } = mountPanel(
    { subjects: [], sources: [] },
    {
      filter: { subjectId: '', book: '教材', chapter: '', knowledge: '' },
      sort: {}
    }
  )

  expect(element.textContent).not.toContain('错因标签')
  expect(element.textContent).not.toContain('时间范围')
  panel.value?.clearAllFilterItems()
  await nextTick()
  expect(model.value.filter).not.toHaveProperty('keyword')
  expect(model.value.filter).not.toHaveProperty('tagIds')
  expect(model.value.filter).not.toHaveProperty('dateRange')
  expect(model.value.sort).toEqual({})
})
