import { attachConsole } from "@tauri-apps/plugin-log";
import { createApp } from "vue";
import "../../styles.css";
import App from "./App.vue";

attachConsole().catch(() => {});
createApp(App).mount("#app");
