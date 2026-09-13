import { quasar, transformAssetUrls } from '@quasar/vite-plugin'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [vue({ template: { transformAssetUrls } }), quasar()],
  test: { environment: 'node', include: ['tests/**/*.test.ts'] }
})
