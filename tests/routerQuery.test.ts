import { expect, it } from 'vitest'
import type { LocationQuery } from 'vue-router'

import {
  parseQuestionListQuery,
  serializeQuestionListQuery
} from '../src/router/types'

it.each([undefined, 'search', 'import'] as const)(
  'round-trips intent %s',
  (intent) => {
    const raw = serializeQuestionListQuery({ intent })
    expect(raw).toEqual(intent ? { [intent]: null } : {})
    expect(parseQuestionListQuery(raw as LocationQuery)).toEqual(
      intent ? { intent } : {}
    )
  }
)

it.each([
  { search: null, import: null },
  { search: '', import: '1' },
  { search: [null, null], import: null }
])('rejects conflicting keys regardless of values: %j', (query) => {
  expect(parseQuestionListQuery(query)).toEqual({})
  expect(serializeQuestionListQuery(parseQuestionListQuery(query))).toEqual({})
})

const invalidQueries: LocationQuery[] = [
  { search: '' },
  { import: '1' },
  { search: [null, null] },
  { focus: 'search' }
]

it.each(invalidQueries)('ignores invalid or legacy signals: %j', (query) => {
  expect(parseQuestionListQuery(query)).toEqual({})
})

it('does not mutate parsed input', () => {
  const input = Object.freeze({ search: null, keep: 'yes' })
  expect(parseQuestionListQuery(input)).toEqual({ intent: 'search' })
  expect(input).toEqual({ search: null, keep: 'yes' })
})
