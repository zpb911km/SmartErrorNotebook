import { createSource, listSources } from '../api/source'
import type { Source } from '../types/source'
import { selectSourceValues, type SourceSelection } from './sourceSelection'

export const materializeSourceSelection = async (
  selection: SourceSelection
): Promise<{ selection: SourceSelection; sourceId: string | null; source?: Source }> => {
  if (selection.kind === 'none') return { selection, sourceId: null }
  if (selection.kind === 'existing') {
    return { selection, sourceId: selection.sourceId }
  }

  const sources = await listSources(selection.subjectId)
  const resolvedSelection = selectSourceValues(
    sources,
    selection.subjectId,
    selection.book,
    selection.chapter,
    selection.knowledge
  )
  if (resolvedSelection.kind === 'existing') {
    return {
      selection: resolvedSelection,
      sourceId: resolvedSelection.sourceId,
      source: sources.find((source) => source.id === resolvedSelection.sourceId)
    }
  }

  const source = await createSource({
    subjectId: selection.subjectId,
    book: selection.book,
    chapter: selection.chapter,
    knowledge: selection.knowledge
  })
  return {
    selection: { kind: 'existing', sourceId: source.id },
    sourceId: source.id,
    source
  }
}
