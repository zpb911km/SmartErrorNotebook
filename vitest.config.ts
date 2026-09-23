import path from 'node:path'

import { quasar, transformAssetUrls } from '@quasar/vite-plugin'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  resolve: { alias: { '@': path.join(import.meta.dirname, 'src') } },
  plugins: [vue({ template: { transformAssetUrls } }), quasar()],
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
    typecheck: {
      enabled: true,
      include: ['tests/**/*.test-d.ts'],
      checker: 'vue-tsc',
      build: true,
      tsconfig: './tsconfig.vitest.json'
    }
  }
})
