import { onScopeDispose, reactive, watch } from 'vue'

import { listQuestions, listSrsData } from '@/api'
import {
  buildLibraryListRequest,
  type LibraryListCriteria,
  sameLibraryNonKeywordCriteria,
  sameLibraryServerCriteria,
  snapshotLibraryServerCriteria
} from '@/services/libraryList'
import {
  loadQuestionMetadata,
  projectQuestions,
  type QuestionMetadata
} from '@/services/questionQueries'
import type { Source, Subject, Tag } from '@/types'
import type { QuestionView } from '@/types/questionView'
import { createLatestTask } from '@/utils/latestTask'

export interface QuestionLibraryState {
  status: 'loading' | 'ready' | 'error'
  questions: QuestionView[]
  subjects: Subject[]
  sources: Source[]
  tags: Tag[]
}

function waitForDelay(signal: AbortSignal, milliseconds: number) {
  return new Promise<void>((resolve) => {
    const onAbort = () => {
      clearTimeout(timer)
      resolve()
    }
    const timer = setTimeout(() => {
      signal.removeEventListener('abort', onAbort)
      resolve()
    }, milliseconds)
    signal.addEventListener('abort', onAbort, { once: true })
  })
}

/** Owns one question-list page's request lifecycle and metadata cache. */
export function useQuestionLibrary(getModel: () => LibraryListCriteria) {
  const libraryData = reactive<QuestionLibraryState>({
    status: 'loading',
    questions: [],
    subjects: [],
    sources: [],
    tags: []
  })
  let metadataPromise: Promise<QuestionMetadata> | undefined
  let disposed = false

  function getMetadata(): Promise<QuestionMetadata> {
    if (!metadataPromise) {
      const pending = loadQuestionMetadata()
      metadataPromise = pending
      void pending.then(
        (metadata) => {
          if (disposed || metadataPromise !== pending) return
          libraryData.subjects = metadata.subjects
          libraryData.sources = metadata.sources
          libraryData.tags = metadata.tags
        },
        () => {
          if (metadataPromise === pending) metadataPromise = undefined
        }
      )
    }
    return metadataPromise
  }

  const query = createLatestTask(
    async (delayMs: number, signal: AbortSignal) => {
      libraryData.status = 'loading'
      if (delayMs) await waitForDelay(signal, delayMs)
      signal.throwIfAborted()

      const now = new Date()
      try {
        const [page, metadata, srs] = await Promise.all([
          listQuestions(buildLibraryListRequest(getModel(), now)),
          getMetadata(),
          listSrsData(now.toISOString())
        ])
        signal.throwIfAborted()
        libraryData.questions = projectQuestions(page.items, {
          ...metadata,
          srs
        })
        libraryData.status = 'ready'
      } catch (error) {
        if (signal.aborted) return
        console.error('获取数据失败:', error)
        libraryData.questions = []
        libraryData.status = 'error'
      }
    }
  )

  watch(
    () => snapshotLibraryServerCriteria(getModel()),
    (current, previous) => {
      if (previous && sameLibraryServerCriteria(current, previous)) return
      const delayMs =
        previous && sameLibraryNonKeywordCriteria(current, previous) ? 250 : 0
      void query(delayMs)
    },
    { immediate: true }
  )

  function refresh(options: { invalidateMetadata?: boolean } = {}) {
    if (options.invalidateMetadata) {
      metadataPromise = undefined
      libraryData.subjects = []
      libraryData.sources = []
      libraryData.tags = []
    }
    return query(0)
  }

  onScopeDispose(() => {
    disposed = true
    query.cancel()
  })

  return { libraryData, refresh }
}
