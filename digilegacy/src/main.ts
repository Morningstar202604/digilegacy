import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import './assets/main.css';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

// 兜底：渲染期异常不应把整页变成白屏（日志保留，便于用户反馈时定位）
app.config.errorHandler = (err, _instance, info) => {
  console.error('[render-error]', info, err);
};

app.mount('#app');
