<template>
  <div class="filter-nav-container">
    <nav
      ref="navRef"
      class="filter-nav"
      :class="{ open: isExpanded }"
      :style="{ backgroundColor: baseColor }"
    >
      <!-- 顶部栏：汉堡菜单 + 标题 + 统计 -->
      <div class="filter-nav-top">
        <div
          class="hamburger-menu"
          :class="{ open: isHamburgerOpen }"
          @click="toggleMenu"
          role="button"
          :aria-label="isExpanded ? '收起筛选' : '展开筛选'"
          :aria-expanded="isExpanded"
          tabindex="0"
          @keydown.enter.prevent="toggleMenu"
          @keydown.space.prevent="toggleMenu"
        >
          <div class="hamburger-line" />
          <div class="hamburger-line" />
        </div>

        <div class="filter-title">
          <slot name="title">筛选条件</slot>
        </div>

        <div class="filter-stats" v-if="totalCount !== undefined">
          <span class="stat-badge">{{ totalCount }} 题</span>
        </div>
      </div>

      <!-- 筛选卡片区 -->
      <div class="filter-nav-content" :aria-hidden="!isExpanded">
        <div
          v-for="(item, idx) in items"
          :key="item.label"
          class="filter-card"
          :ref="el => setCardRef(el, idx)"
          :style="{ backgroundColor: item.bgColor, color: item.textColor }"
        >
          <div class="filter-card-label">{{ item.label }}</div>
          <div class="filter-card-body">
            <slot :name="'body-' + item.key" :item="item">
              <!-- 默认插槽内容 -->
              <div
                v-for="opt in item.options"
                :key="opt.value"
                class="filter-option"
                :class="{ active: isActive(item.key, opt.value) }"
                @click="handleSelect(item.key, opt.value)"
              >
                {{ opt.label }}
              </div>
            </slot>
          </div>
        </div>
      </div>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { gsap } from 'gsap'

export interface FilterNavItem {
  label: string
  key: string
  bgColor: string
  textColor: string
  options: { label: string; value: string }[]
}

export interface FilterNavState {
  [key: string]: string | string[]
}

const props = withDefaults(defineProps<{
  items: FilterNavItem[]
  modelValue?: FilterNavState
  totalCount?: number
  baseColor?: string
  menuColor?: string
  ease?: string
}>(), {
  baseColor: 'var(--card-bg)',
  ease: 'power3.out'
})

const emit = defineEmits<{
  (e: 'update:modelValue', val: FilterNavState): void
  (e: 'select', key: string, value: string | string[]): void
}>()

const isHamburgerOpen = ref(false)
const isExpanded = ref(false)
const navRef = ref<HTMLElement | null>(null)
const cardsRef = ref<HTMLElement[]>([])
const tlRef = ref<gsap.core.Timeline | null>(null)

const localState = ref<FilterNavState>(props.modelValue || {})

watch(() => props.modelValue, (val) => {
  if (val) localState.value = val
}, { deep: true })

function isActive(key: string, value: string): boolean {
  const v = localState.value[key]
  if (Array.isArray(v)) return v.includes(value)
  return v === value
}

function handleSelect(key: string, value: string) {
  // 单选模式
  if (localState.value[key] === value) {
    const newState = { ...localState.value }
    delete newState[key]
    localState.value = newState
    emit('update:modelValue', newState)
    emit('select', key, '')
  } else {
    const newState = { ...localState.value, [key]: value }
    localState.value = newState
    emit('update:modelValue', newState)
    emit('select', key, value)
  }
}

function setCardRef(el: any, idx: number) {
  if (el) cardsRef.value[idx] = el as HTMLElement
}

function calculateHeight(): number {
  const navEl = navRef.value
  if (!navEl) return 260

  const isMobile = window.innerWidth <= 768
  if (isMobile) {
    const contentEl = navEl.querySelector('.filter-nav-content') as HTMLElement | null
    if (contentEl) {
      const wasVisible = contentEl.style.visibility
      const wasPointerEvents = contentEl.style.pointerEvents
      const wasPosition = contentEl.style.position
      const wasHeight = contentEl.style.height

      contentEl.style.visibility = 'visible'
      contentEl.style.pointerEvents = 'auto'
      contentEl.style.position = 'static'
      contentEl.style.height = 'auto'
      contentEl.offsetHeight

      const topBar = 60
      const padding = 16
      const contentHeight = contentEl.scrollHeight

      contentEl.style.visibility = wasVisible
      contentEl.style.pointerEvents = wasPointerEvents
      contentEl.style.position = wasPosition
      contentEl.style.height = wasHeight

      return topBar + contentHeight + padding
    }
  }
  return 260
}

function createTimeline(): gsap.core.Timeline | null {
  const navEl = navRef.value
  if (!navEl) return null

  gsap.set(navEl, { height: 60, overflow: 'hidden' })
  gsap.set(cardsRef.value, { y: 50, opacity: 0 })

  const tl = gsap.timeline({ paused: true })

  tl.to(navEl, {
    height: calculateHeight(),
    duration: 0.4,
    ease: props.ease
  })

  tl.to(cardsRef.value, {
    y: 0,
    opacity: 1,
    duration: 0.4,
    ease: props.ease,
    stagger: 0.08
  }, '-=0.1')

  return tl
}

function toggleMenu() {
  const tl = tlRef.value
  if (!tl) return

  if (!isExpanded.value) {
    isHamburgerOpen.value = true
    isExpanded.value = true
    tl.play(0)
  } else {
    isHamburgerOpen.value = false
    tl.eventCallback('onReverseComplete', () => {
      isExpanded.value = false
    })
    tl.reverse()
  }
}

// ===== 初始化时间线 =====
function initTimeline() {
  const tl = createTimeline()
  tlRef.value = tl
}

// ===== 响应式 resize 处理 =====
watch(isExpanded, () => {
  // 等 DOM 更新后再处理 resize
  nextTick(() => {
    if (!tlRef.value) return
    if (isExpanded.value) {
      const newHeight = calculateHeight()
      gsap.set(navRef.value, { height: newHeight })
      tlRef.value.kill()
      const newTl = createTimeline()
      if (newTl) {
        newTl.progress(1)
        tlRef.value = newTl
      }
    }
  })
})

onMounted(() => {
  initTimeline()

  const handleResize = () => {
    if (!tlRef.value) return
    if (isExpanded.value) {
      const newHeight = calculateHeight()
      gsap.set(navRef.value, { height: newHeight })
      tlRef.value.kill()
      const newTl = createTimeline()
      if (newTl) {
        newTl.progress(1)
        tlRef.value = newTl
      }
    } else {
      tlRef.value.kill()
      const newTl = createTimeline()
      if (newTl) tlRef.value = newTl
    }
  }

  window.addEventListener('resize', handleResize)
  onBeforeUnmount(() => {
    window.removeEventListener('resize', handleResize)
    tlRef.value?.kill()
    tlRef.value = null
  })
})
</script>

<style scoped>
.filter-nav-container {
  width: 100%;
  box-sizing: border-box;
  margin-bottom: 16px;
}

.filter-nav {
  display: block;
  height: 60px;
  padding: 0;
  background-color: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  box-shadow: var(--shadow-md);
  position: relative;
  overflow: hidden;
  will-change: height;
}

.filter-nav-top {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.45rem 0.55rem 1.1rem;
  z-index: 2;
}

.hamburger-menu {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  gap: 6px;
  color: var(--text-primary);
}

.hamburger-menu:hover .hamburger-line {
  opacity: 0.75;
}

.hamburger-line {
  width: 30px;
  height: 2px;
  background-color: currentColor;
  transition:
    transform 0.25s ease,
    opacity 0.2s ease,
    margin 0.3s ease;
  transform-origin: 50% 50%;
}

.hamburger-menu.open .hamburger-line:first-child {
  transform: translateY(4px) rotate(45deg);
}

.hamburger-menu.open .hamburger-line:last-child {
  transform: translateY(-4px) rotate(-45deg);
}

.filter-title {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

.filter-stats {
  display: flex;
  align-items: center;
  padding-right: 0.5rem;
}

.stat-badge {
  background: var(--primary-color);
  color: white;
  font-size: 13px;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 999px;
}

.filter-nav-content {
  position: absolute;
  left: 0;
  right: 0;
  top: 60px;
  bottom: 0;
  padding: 0.5rem;
  display: flex;
  align-items: flex-end;
  gap: 12px;
  visibility: hidden;
  pointer-events: none;
  z-index: 1;
}

.filter-nav.open .filter-nav-content {
  visibility: visible;
  pointer-events: auto;
}

.filter-card {
  height: 100%;
  flex: 1 1 0;
  min-width: 0;
  border-radius: calc(0.75rem - 0.2rem);
  position: relative;
  display: flex;
  flex-direction: column;
  padding: 12px 16px;
  gap: 8px;
  user-select: none;
}

.filter-card-label {
  font-weight: 600;
  font-size: 15px;
  letter-spacing: -0.3px;
  opacity: 0.9;
}

.filter-card-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.filter-option {
  font-size: 14px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
  opacity: 0.7;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.filter-option:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.15);
}

.filter-option.active {
  opacity: 1;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.2);
}

@media (max-width: 768px) {
  .filter-nav-container {
    width: 100%;
  }

  .filter-nav-top {
    padding: 0.5rem 1rem;
    justify-content: space-between;
  }

  .hamburger-menu {
    order: 2;
  }

  .filter-title {
    position: static;
    transform: none;
    order: 1;
  }

  .filter-stats {
    display: none;
  }

  .filter-nav-content {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 0.5rem;
    bottom: 0;
    justify-content: flex-start;
  }

  .filter-card {
    height: auto;
    min-height: 60px;
    flex: 1 1 auto;
    max-height: none;
  }
}
</style>
