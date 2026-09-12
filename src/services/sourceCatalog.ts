import type { Source } from '../types'

const unique = (values: Array<string | null>) =>
  [...new Set(values.filter((value): value is string => value !== null))].sort()

export const sourceBooks = (sources: Source[]) =>
  unique(sources.map((source) => source.book))
export const sourceChapters = (sources: Source[], book: string) =>
  unique(
    sources
      .filter((source) => source.book === book)
      .map((source) => source.chapter)
  )
export const sourceKnowledges = (
  sources: Source[],
  book: string,
  chapter: string
) =>
  unique(
    sources
      .filter((source) => source.book === book && source.chapter === chapter)
      .map((source) => source.knowledge)
  )

/** Derive cascade choices from the page's loaded resources, without extra IPC. */
export function createSourceCatalog(readSources: () => Source[]) {
  const selected = (subjectId?: string) =>
    readSources().filter(
      (source) => !subjectId || source.subjectId === subjectId
    )
  return {
    getBooks: (subjectId?: string) => sourceBooks(selected(subjectId)),
    getChapters: (book: string, subjectId?: string) =>
      sourceChapters(selected(subjectId), book),
    getKnowledges: (book: string, chapter: string, subjectId?: string) =>
      sourceKnowledges(selected(subjectId), book, chapter)
  }
}
