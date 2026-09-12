import { onScopeDispose } from 'vue'

/** Invalidates older requests, including responses arriving after unmount. */
export function useLatestRequest() {
  let version = 0
  let disposed = false
  onScopeDispose(() => {
    disposed = true
    version++
  })
  return () => {
    const current = ++version
    return () => !disposed && current === version
  }
}
