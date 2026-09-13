import { expect, it } from 'vitest'
import { effectScope } from 'vue'

import { useLatestRequest } from '../src/composables/useLatestRequest'

it('invalidates an older response and all responses after disposal', () => {
  const scope = effectScope()
  const begin = scope.run(() => useLatestRequest())!
  const oldResponse = begin()
  const latestResponse = begin()
  expect(oldResponse()).toBe(false)
  expect(latestResponse()).toBe(true)
  scope.stop()
  expect(latestResponse()).toBe(false)
  expect(begin()()).toBe(false)
})
