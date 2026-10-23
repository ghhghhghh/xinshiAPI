import { createApp } from "vue";
import { createPinia } from "pinia";
import "@fontsource-variable/noto-sans-sc";
import "vue-sonner/style.css";
import "./assets/main.css";
import App from "./App.vue";

createApp(App).use(createPinia()).mount("#app");
