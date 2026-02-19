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

app.use(router);
app.mount("#app");
