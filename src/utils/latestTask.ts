export type AbortableAsyncTask<Args extends unknown[], Result> = (
  ...args: [...Args, signal: AbortSignal]
) => Promise<Result>

export interface LatestTask<Args extends unknown[], Result> {
  (...args: Args): Promise<Result | undefined>
  cancel(): void
}

/** Allows only the latest invocation to produce a result. */
export function createLatestTask<Args extends unknown[], Result>(
  task: AbortableAsyncTask<Args, Result>
): LatestTask<Args, Result> {
  let current: AbortController | undefined

  const run = async (...args: Args): Promise<Result | undefined> => {
    current?.abort()
    const controller = new AbortController()
    current = controller

    try {
      const result = await task(...args, controller.signal)
      return controller.signal.aborted ? undefined : result
    } catch (error) {
      if (controller.signal.aborted) return undefined
      throw error
    } finally {
      if (current === controller) current = undefined
    }
  }

  return Object.assign(run, {
    cancel() {
      current?.abort()
      current = undefined
    }
  })
}
