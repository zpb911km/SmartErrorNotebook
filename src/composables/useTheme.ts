import { Dark } from 'quasar'
import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark' | 'system'
const saved = localStorage.getItem('theme')
export const theme = ref<Theme>(
  saved === 'light' || saved === 'dark' ? saved : 'system'
)

export function setTheme(value: Theme) {
  theme.value = value
  localStorage.setItem('theme', value)
  Dark.set(value === 'system' ? 'auto' : value === 'dark')
}

export function initializeTheme() {
  setTheme(theme.value)
  const dispose = watch(
    () => Dark.isActive,
    (dark) => {
      document.body.classList.toggle('dark-theme', dark)
      document.body.classList.toggle('light-theme', !dark)
    },
    { immediate: true }
  )
  return { dispose }
}
