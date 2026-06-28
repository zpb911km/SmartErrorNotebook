import { ref, watch, onUnmounted, type Ref } from 'vue'

/**
 * 数字滚动计数动画
 * @param sourceRef 目标数值 ref（变化时自动重新动画）
 * @param duration 动画持续毫秒数（默认 1000ms）
 */
export function useCountUp(sourceRef: Ref<number>, duration = 1000) {
  const displayValue = ref(0)
  let rafId: number | null = null
  let started = false

  function animate(from: number, to: number) {
    if (rafId) cancelAnimationFrame(rafId)
    if (from === to) {
      displayValue.value = to
      return
    }

    const startTime = performance.now()

    function step(currentTime: number) {
      const elapsed = currentTime - startTime
      const progress = Math.min(elapsed / duration, 1)
      // easeOutCubic
      const eased = 1 - Math.pow(1 - progress, 3)
      displayValue.value = Math.round(from + (to - from) * eased)

      if (progress < 1) {
        rafId = requestAnimationFrame(step)
      }
    }

    rafId = requestAnimationFrame(step)
  }

  // 首次值变化时触发（从 0 到目标），后续变化也重新动画
  watch(sourceRef, (newVal) => {
    if (newVal > 0 || started) {
      animate(started ? 0 : 0, newVal)
      started = true
    }
  })

  onUnmounted(() => {
    if (rafId) cancelAnimationFrame(rafId)
  })

  return { displayValue }
}
