import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import MarkdownTextarea from './components/MarkdownTextarea.vue'
import Icon from './components/Icon.vue'
import scrollReveal from './directives/scrollReveal'
import ripple from './directives/ripple'
import './styles/theme.css'
import './styles/global.css'

const app = createApp(App)
app.use(router)
app.component('MarkdownTextarea', MarkdownTextarea)
app.component('Icon', Icon)
app.directive('scroll-reveal', scrollReveal)
app.directive('ripple', ripple)
app.mount('#app')
