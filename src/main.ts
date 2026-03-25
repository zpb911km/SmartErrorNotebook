import { createApp, ref, provide } from "vue";
import App from "./App.vue";
import router from "./router";
import "./styles/theme.css";
import "./styles/global.css";

const app = createApp(App);

// 创建全局AI状态
const aiState = ref({
  enabled: true,
  remainingTokens: 1250,
  modelName: 'GPT-4'
});

// 提供全局状态
app.provide('aiState', aiState);

// 初始化主题
const initTheme = () => {
  const savedTheme = localStorage.getItem('theme');
  let themeValue = savedTheme || 'system';
  
  // 移除所有主题类
  document.body.classList.remove('light-theme', 'dark-theme');
  
  if (themeValue === 'light') {
    document.body.classList.add('light-theme');
  } else if (themeValue === 'dark') {
    document.body.classList.add('dark-theme');
  } else if (themeValue === 'system') {
    // 跟随系统
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.body.classList.add('dark-theme');
    } else {
      document.body.classList.add('light-theme');
    }
  }
};

// 初始化主题
initTheme();

// 监听系统主题变化
if (window.matchMedia) {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme === 'system') {
      initTheme();
    }
  });
}

app.use(router);
app.mount("#app");
