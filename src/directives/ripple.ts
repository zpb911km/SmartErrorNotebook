/**
 * v-ripple 指令
 * 点击时产生水波纹扩散动画
 * 用法：v-ripple 或 v-ripple="{ color: 'rgba(255,255,255,0.4)', duration: 600 }"
 */
export default {
  mounted(el: HTMLElement, binding: any) {
    const opts = binding.value || {}
    const color = opts.color ?? 'rgba(255,255,255,0.35)'
    const duration = opts.duration ?? 600

    el.style.position = 'relative'
    el.style.overflow = 'hidden'

    function onClick(e: MouseEvent) {
      const rect = el.getBoundingClientRect()
      const size = Math.max(rect.width, rect.height) * 2
      const x = e.clientX - rect.left - size / 2
      const y = e.clientY - rect.top - size / 2

      const ripple = document.createElement('span')
      ripple.style.cssText = `
        position: absolute;
        left: ${x}px;
        top: ${y}px;
        width: ${size}px;
        height: ${size}px;
        border-radius: 50%;
        background: ${color};
        pointer-events: none;
        transform: scale(0);
        animation: ripple-anim ${duration}ms ease-out forwards;
        z-index: -1;
      `
      el.appendChild(ripple)
      ripple.addEventListener('animationend', () => ripple.remove())
    }

    el.addEventListener('click', onClick)
  }
}
