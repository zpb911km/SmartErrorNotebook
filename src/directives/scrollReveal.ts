import type { DirectiveBinding } from 'vue'

/**
 * v-scroll-reveal 指令
 * 元素进入视口时从底部渐入上移
 * 用法：v-scroll-reveal 或 v-scroll-reveal="{ delay: 100, offset: 30 }"
 */
export default {
  mounted(el: HTMLElement, binding: DirectiveBinding) {
    const opts = binding.value || {}
    const delay = opts.delay ?? 0
    const offset = opts.offset ?? 24
    const duration = opts.duration ?? 600

    // 避免闪动：初始透明 + 下移
    el.style.opacity = '0'
    el.style.transform = `translateY(${offset}px)`
    el.style.transition = `all ${duration}ms cubic-bezier(0.16, 1, 0.3, 1)`

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setTimeout(() => {
            el.style.opacity = '1'
            el.style.transform = 'translateY(0)'
          }, delay)
          observer.unobserve(el)
        }
      },
      { threshold: 0.1 }
    )
    observer.observe(el)
  }
}
