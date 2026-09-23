import { vueTsConfigs, withVueTs } from '@vue/eslint-config-typescript'
import prettierConfig from 'eslint-config-prettier'
import simpleImportSort from 'eslint-plugin-simple-import-sort'
import pluginVue from 'eslint-plugin-vue'

const config: ReturnType<typeof withVueTs> = withVueTs(
  {
    ignores: [
      'src-tauri/gen/**',
      'src-tauri/target/**',
      'node_modules/**',
      'dist/**',
      '.tsbuild/**',
      'stats.html',
      'test-results/**',
      'playwright-report/**'
    ]
  },
  pluginVue.configs['flat/recommended'],
  vueTsConfigs.recommended,

  {
    plugins: {
      'simple-import-sort': simpleImportSort
    },

    rules: {
      'simple-import-sort/imports': 'error',
      'simple-import-sort/exports': 'error'
    }
  },

  prettierConfig
)

export default config
