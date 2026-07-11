import "./assets/main.css";

import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import naive from "naive-ui";
// markdown编辑器
import 'vditor/dist/index.css'
// 通用字体
import "vfonts/Lato.css";
import { useUserStore } from "./stores/user";

// msw
// if (import.meta.env.DEV) {
//   const { worker } = await import("./mocks/browser.ts");
//   await worker.start({
//     onUnhandledRequest: "bypass", // 未 mock 的请求直接放行
//   });
// }

const app = createApp(App);
const pinia = createPinia();

// Piania、Router、Naive-ui
app.use(pinia);
useUserStore(pinia).initFromStorage();
app.use(router);
app.use(naive);

app.mount("#app");

