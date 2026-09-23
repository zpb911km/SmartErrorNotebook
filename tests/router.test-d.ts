import { expectTypeOf, test } from 'vitest'
import { useRoute, useRouter } from 'vue-router'

import {
  goHome,
  goQuestionDetail,
  goQuestionList,
  goReviewPlan
} from '../src/router'
import { serializeQuestionListQuery } from '../src/router/types'

// This file is collected only by Vitest's typecheck runner.
test('route type contract', () => {
  const router = useRouter()
  const route = useRoute('question-detail')
  expectTypeOf(route.params.id).toEqualTypeOf<string>()
  router.push({ name: 'review-plan' })
  goQuestionDetail('one', { force: true, replace: false })
  goQuestionList({ intent: 'search' }, { replace: true })
  goQuestionList({}, { force: true })
  goQuestionList()
  serializeQuestionListQuery({ intent: 'import' })
  // @ts-expect-error Unknown query fields are not supported.
  goQuestionList({ keep: 'yes' })
  // @ts-expect-error The serializer no longer accepts a base query.
  serializeQuestionListQuery({}, { keep: 'yes' })
  goHome({ force: false })
  goReviewPlan({ replace: true })
  // @ts-expect-error String intents are no longer accepted.
  goQuestionList('search')
  // @ts-expect-error Unknown intents are invalid.
  goQuestionList('unknown')
  // @ts-expect-error Navigation options must be boolean.
  goHome({ force: 'yes' })
  // @ts-expect-error Intent does not belong in navigation options.
  goQuestionList(undefined, { intent: 'search' })
  // @ts-expect-error Unknown route names must not be accepted.
  router.push({ name: 'unknown-route' })
  // @ts-expect-error IDs must be strings.
  router.push({ name: 'question-detail', params: { id: 1 } })
  // Vue Router can reuse current params; our cross-page operation cannot.
  // @ts-expect-error An explicit ID is required.
  goQuestionDetail()
  // @ts-expect-error Array IDs are invalid.
  goQuestionDetail(['one'])
  // @ts-expect-error Numeric IDs are invalid.
  goQuestionDetail(1)
})
