import { expect, it } from 'vitest'

import { createSourceCatalog } from '../src/services/sourceCatalog'
import type { Source } from '../src/types'

it('derives subject-scoped cascades synchronously from refreshed resource data', () => {
  let sources: Source[] = [
    {
      id: 'a',
      subjectId: 'math',
      book: 'Book',
      chapter: 'One',
      knowledge: 'Algebra',
      createdAt: '',
      updatedAt: ''
    },
    {
      id: 'b',
      subjectId: 'math',
      book: 'Book',
      chapter: 'One',
      knowledge: 'Geometry',
      createdAt: '',
      updatedAt: ''
    },
    {
      id: 'c',
      subjectId: 'other',
      book: 'Other',
      chapter: 'Two',
      knowledge: null,
      createdAt: '',
      updatedAt: ''
    }
  ]
  const catalog = createSourceCatalog(() => sources)
  expect(catalog.getBooks('math')).toEqual(['Book'])
  expect(catalog.getChapters('Book', 'math')).toEqual(['One'])
  expect(catalog.getKnowledges('Book', 'One', 'math')).toEqual([
    'Algebra',
    'Geometry'
  ])
  sources = []
  expect(catalog.getBooks('math')).toEqual([])
})
