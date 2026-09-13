import '@quasar/extras/material-icons/material-icons.css'
import 'quasar/src/css/index.sass'
import 'highlight.js/styles/github-dark.css'
import 'katex/dist/katex.min.css'
import './styles/theme.css'
import './styles/global.css'

import { Quasar } from 'quasar'
import { createApp } from 'vue'

import App from './App.vue'
import Icon from './components/Icon.vue'
import MarkdownTextarea from './components/MarkdownTextarea.vue'
import { quasarOptions } from './quasar'
import router from './router'

const app = createApp(App)
app.use(router)
app.use(Quasar, quasarOptions)
app.component('MarkdownTextarea', MarkdownTextarea)
app.component('Icon', Icon)
app.mount('#app')
