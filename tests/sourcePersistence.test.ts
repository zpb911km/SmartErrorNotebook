import { beforeEach, expect, it, vi } from 'vitest'

import { createSource, listSources } from '../src/api/source'
import { materializeSourceSelection } from '../src/services/sourcePersistence'
import type { SourceSelection } from '../src/services/sourceSelection'
import type { Source } from '../src/types'

vi.mock('../src/api/source', () => ({
  createSource: vi.fn(),
  listSources: vi.fn()
}))
const selection: SourceSelection = {
  kind: 'new',
  subjectId: 'subject',
  book: null,
  chapter: null,
  knowledge: null
}
const source: Source = {
  id: 'source',
  subjectId: 'subject',
  book: null,
  chapter: null,
  knowledge: null,
  createdAt: '2026-01-01',
  updatedAt: '2026-01-01'
}
beforeEach(() => {
  vi.resetAllMocks()
})

it('reuses a matching source and creates no source for unclassified questions', async () => {
  vi.mocked(listSources).mockResolvedValue([source])
  expect((await materializeSourceSelection(selection)).sourceId).toBe('source')
  expect(
    (await materializeSourceSelection({ kind: 'none' })).sourceId
  ).toBeNull()
  expect(createSource).not.toHaveBeenCalled()
})

it('coalesces concurrent matching drafts but queries again after completion', async () => {
  vi.mocked(listSources).mockResolvedValue([])
  vi.mocked(createSource).mockResolvedValue(source)
  const [first, second] = await Promise.all([
    materializeSourceSelection(selection),
    materializeSourceSelection({ ...selection })
  ])
  expect(first.sourceId).toBe(second.sourceId)
  expect(createSource).toHaveBeenCalledTimes(1)
  vi.mocked(listSources).mockResolvedValue([source])
  await materializeSourceSelection(selection)
  expect(listSources).toHaveBeenCalledTimes(2)
  expect(createSource).toHaveBeenCalledTimes(1)
})

it('releases failed resolutions so they can be retried', async () => {
  vi.mocked(listSources)
    .mockRejectedValueOnce(new Error('storage'))
    .mockResolvedValueOnce([source])
  await expect(materializeSourceSelection(selection)).rejects.toThrow('storage')
  expect((await materializeSourceSelection(selection)).sourceId).toBe('source')
})
