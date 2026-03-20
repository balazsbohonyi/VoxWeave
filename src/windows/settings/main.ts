import { attachConsole } from "@tauri-apps/plugin-log";
import { createApp } from "vue";
import "../../styles.css";
import App from "./App.vue";

await attachConsole();
createApp(App).mount("#app");
