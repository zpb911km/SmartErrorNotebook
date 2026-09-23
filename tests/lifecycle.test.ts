import { expect, it } from 'vitest'
import { effectScope } from 'vue'

import { useLatestRequest } from '../src/composables/useLatestRequest'
import { createLatestTask } from '../src/utils/latestTask'

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

it('aborts an older task and returns only the latest result', async () => {
  const releases: Array<() => void> = []
  const signals: AbortSignal[] = []
  const task = createLatestTask(
    async (value: string, signal: AbortSignal): Promise<string> => {
      signals.push(signal)
      await new Promise<void>((resolve) => releases.push(resolve))
      signal.throwIfAborted()
      return value
    }
  )

  const first = task('first')
  const second = task('second')
  expect(signals[0].aborted).toBe(true)
  expect(signals[1].aborted).toBe(false)

  releases[0]()
  releases[1]()
  await expect(first).resolves.toBeUndefined()
  await expect(second).resolves.toBe('second')
})

it('supports explicit cancellation and propagates current task failures', async () => {
  let release!: () => void
  const pendingTask = createLatestTask(async (signal: AbortSignal) => {
    await new Promise<void>((resolve) => {
      release = resolve
    })
    signal.throwIfAborted()
    return 'complete'
  })
  const pending = pendingTask()
  pendingTask.cancel()
  release()
  await expect(pending).resolves.toBeUndefined()

  const failure = new Error('current failure')
  const failingTask = createLatestTask(async (signal: AbortSignal) => {
    expect(signal.aborted).toBe(false)
    throw failure
  })
  await expect(failingTask()).rejects.toBe(failure)
})
