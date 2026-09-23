import type { ListQuestionsRequest } from '@/types'

export interface LibraryListCriteria {
  filter: {
    keyword?: string
    book?: string
    chapter?: string
    knowledge?: string
    tagIds?: readonly string[]
    dateRange?: 'all' | '7days' | '30days' | '90days'
  }
  sort: { mastery?: 'none' | 'asc' | 'desc' }
}

export interface LibraryServerCriteria {
  keyword: string
  book: string
  chapter: string
  knowledge: string
  tagIds: string[]
  dateRange: LibraryListCriteria['filter']['dateRange']
  mastery: LibraryListCriteria['sort']['mastery']
}

export function snapshotLibraryServerCriteria(
  model: LibraryListCriteria
): LibraryServerCriteria {
  const { filter, sort } = model
  return {
    keyword: filter.keyword ?? '',
    book: filter.book ?? '',
    chapter: filter.chapter ?? '',
    knowledge: filter.knowledge ?? '',
    tagIds: [...(filter.tagIds ?? [])],
    dateRange: filter.dateRange,
    mastery: sort.mastery
  }
}

export function sameLibraryServerCriteria(
  current: LibraryServerCriteria,
  previous: LibraryServerCriteria
): boolean {
  return (
    current.keyword === previous.keyword &&
    sameLibraryNonKeywordCriteria(current, previous)
  )
}

export function sameLibraryNonKeywordCriteria(
  current: LibraryServerCriteria,
  previous: LibraryServerCriteria
): boolean {
  return (
    current.book === previous.book &&
    current.chapter === previous.chapter &&
    current.knowledge === previous.knowledge &&
    current.dateRange === previous.dateRange &&
    current.mastery === previous.mastery &&
    current.tagIds.length === previous.tagIds.length &&
    current.tagIds.every((id, index) => id === previous.tagIds[index])
  )
}

const dateRangeDays = { '7days': 7, '30days': 30, '90days': 90 } as const

export function buildLibraryListRequest(
  model: LibraryListCriteria,
  now = new Date()
): ListQuestionsRequest {
  const { filter, sort } = model
  const days =
    filter.dateRange && filter.dateRange !== 'all'
      ? dateRangeDays[filter.dateRange]
      : undefined
  const masterySort =
    sort.mastery === 'asc'
      ? 'MASTERY_ASC'
      : sort.mastery === 'desc'
        ? 'MASTERY_DESC'
        : undefined

  return {
    filter: {
      ...(filter.keyword ? { keyword: filter.keyword } : {}),
      ...(filter.book ? { book: filter.book } : {}),
      ...(filter.chapter ? { chapter: filter.chapter } : {}),
      ...(filter.knowledge ? { knowledge: filter.knowledge } : {}),
      ...(filter.tagIds?.length ? { tagIds: [...filter.tagIds] } : {}),
      ...(days
        ? {
            updatedSince: new Date(
              now.getTime() - days * 86400000
            ).toISOString()
          }
        : {})
    },
    sort: masterySort
      ? [masterySort, 'UPDATED_AT_DESC', 'ID_ASC']
      : ['UPDATED_AT_DESC']
  }
}
