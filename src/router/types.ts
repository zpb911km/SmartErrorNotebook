import type { LocationQuery, LocationQueryRaw } from 'vue-router'

import type { RouteNamedMap } from './catalog'

export type QuestionListQuery = { intent?: QuestionListIntent }
export type QuestionListIntent = 'search' | 'import'

function hasQuestionListQueryConflict(query: LocationQuery): boolean {
  return (
    Object.prototype.hasOwnProperty.call(query, 'search') &&
    Object.prototype.hasOwnProperty.call(query, 'import')
  )
}

export function parseQuestionListQuery(
  query: LocationQuery
): QuestionListQuery {
  if (hasQuestionListQueryConflict(query)) return {}
  if (query.search === null) return { intent: 'search' }
  if (query.import === null) return { intent: 'import' }
  return {}
}

// Serialize only the typed, mutually exclusive intent state.
export function serializeQuestionListQuery(
  query: QuestionListQuery
): LocationQueryRaw {
  const result: LocationQueryRaw = {}
  if (query.intent) result[query.intent] = null
  return result
}

declare module 'vue-router' {
  interface TypesConfig {
    RouteNamedMap: RouteNamedMap
  }
}
