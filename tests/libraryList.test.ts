import { describe, expect, it } from 'vitest'

import {
  buildLibraryListRequest,
  sameLibraryNonKeywordCriteria,
  sameLibraryServerCriteria,
  snapshotLibraryServerCriteria
} from '../src/services/libraryList'

describe('library list request', () => {
  it('uses one supplied clock for date filtering and preserves sort precedence', () => {
    const request = buildLibraryListRequest(
      {
        filter: {
          keyword: ' 函数 ',
          tagIds: ['tag'],
          dateRange: '7days'
        },
        sort: { mastery: 'asc' }
      },
      new Date('2026-09-22T12:00:00Z')
    )

    expect(request).toEqual({
      filter: {
        keyword: ' 函数 ',
        tagIds: ['tag'],
        updatedSince: '2026-09-15T12:00:00.000Z'
      },
      sort: ['MASTERY_ASC', 'UPDATED_AT_DESC', 'ID_ASC']
    })
  })

  it('detects in-place tag changes without treating a keyword-only edit as an immediate filter', () => {
    const model = {
      filter: { keyword: 'a', tagIds: ['first'] },
      sort: { mastery: 'none' as const }
    }
    const before = snapshotLibraryServerCriteria(model)
    model.filter.keyword = 'ab'
    const keywordOnly = snapshotLibraryServerCriteria(model)
    expect(sameLibraryServerCriteria(keywordOnly, before)).toBe(false)
    expect(sameLibraryNonKeywordCriteria(keywordOnly, before)).toBe(true)

    model.filter.tagIds.push('second')
    const changedTags = snapshotLibraryServerCriteria(model)
    expect(sameLibraryNonKeywordCriteria(changedTags, keywordOnly)).toBe(false)
    expect(before.tagIds).toEqual(['first'])
  })
})
