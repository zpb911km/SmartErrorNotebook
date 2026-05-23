import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import MarkdownTextarea from './components/MarkdownTextarea.vue'
import './styles/theme.css'
import './styles/global.css'

const app = createApp(App)
app.use(router)
app.component('MarkdownTextarea', MarkdownTextarea)
app.mount('#app')
