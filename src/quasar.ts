import { Dialog, Notify } from 'quasar'
import iconSet from 'quasar/icon-set/material-icons'
import lang from 'quasar/lang/zh-CN'

export const quasarOptions = {
  lang,
  iconSet,
  plugins: { Dialog, Notify },
  config: {
    brand: {
      primary: '#2563eb',
      secondary: '#0d9488',
      accent: '#7c3aed',
      dark: '#172033'
    },
    notify: { position: 'top' as const, timeout: 4000, progress: true }
  }
}
