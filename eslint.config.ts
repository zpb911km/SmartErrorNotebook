import { vueTsConfigs, withVueTs } from '@vue/eslint-config-typescript'
import simpleImportSort from 'eslint-plugin-simple-import-sort'
import pluginVue from 'eslint-plugin-vue'

export default withVueTs(
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'src-tauri/target/**',
      'src-tauri/gen/**',
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
      'vue/multi-word-component-names': [
        'error',
        {
          ignores: [
            'Icon',
            'Add',
            'Home',
            'Manage',
            'Preview',
            'Profile',
            'Settings',
            'Sync'
          ]
        }
      ],
      // Prettier owns layout; retain Vue's semantic checks.
      ...Object.fromEntries(
        Object.entries(pluginVue.rules)
          .filter(([, rule]) => rule.meta?.type === 'layout')
          .map(([name]) => [`vue/${name}`, 'off'])
      ),
      'simple-import-sort/imports': 'error',
      'simple-import-sort/exports': 'error'
    }
  }
)
