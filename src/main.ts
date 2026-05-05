import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import "./style.css";
import App from "./App.vue";
import router from "./router";

// Global error handler
window.addEventListener("error", (event) => {
  console.error("Global error:", event.error);
});

window.addEventListener("unhandledrejection", (event) => {
  console.error("Unhandled promise rejection:", event.reason);
});

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(naive);

app.config.errorHandler = (err, instance, info) => {
  console.error("Vue error:", err);
  console.error("Component:", instance);
  console.error("Info:", info);
};

app.mount("#app");
