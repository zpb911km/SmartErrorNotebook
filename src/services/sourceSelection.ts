import type { Source } from '../types/source'

export type SourceSelection =
  | { kind: 'none' }
  | { kind: 'existing'; sourceId: string }
  | {
      kind: 'new'
      subjectId: string
      book: string | null
      chapter: string | null
      knowledge: string | null
    }

const compareSources = (left: Source, right: Source) =>
  left.createdAt.localeCompare(right.createdAt) ||
  left.id.localeCompare(right.id)

export const selectSourceValues = (
  sources: Source[],
  subjectId: string,
  book: string | null,
  chapter: string | null,
  knowledge: string | null,
  preferredSourceId?: string
): SourceSelection => {
  if (!subjectId) return { kind: 'none' }

  const matches = sources
    .filter(
      (source) =>
        source.subjectId === subjectId &&
        source.book === book &&
        source.chapter === chapter &&
        source.knowledge === knowledge
    )
    .sort(compareSources)
  const existing =
    matches.find((source) => source.id === preferredSourceId) ?? matches[0]

  return existing
    ? { kind: 'existing', sourceId: existing.id }
    : { kind: 'new', subjectId, book, chapter, knowledge }
}
