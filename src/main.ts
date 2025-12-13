import { createApp } from "vue";
import App from "./App.vue";
import "./styles/global.css";
import { initLogger } from "./utils/logger";

initLogger();

createApp(App).mount("#app");
