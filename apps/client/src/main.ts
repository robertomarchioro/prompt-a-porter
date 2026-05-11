import App from "./App.svelte";
import { mount } from "svelte";
import { attachConsole } from "@tauri-apps/plugin-log";

// v0.8.7 Sezione Sviluppo → Debug log: pipe `console.log/info/warn/error`
// del webview → backend tauri-plugin-log → file rolling in app_log_dir.
// Il filtro per livello è applicato lato backend via `log::set_max_level()`
// basato sulla preferenza `debug_log_abilitato`. Quando il toggle è OFF,
// gli error/warn arrivano comunque (utile per diagnosticare crash); con
// ON anche info/debug per troubleshooting completo.
void attachConsole().catch((e) => {
  // Backend non disponibile in dev se Tauri non è avviato: fallback silente,
  // il logging resta solo sulla console nativa del browser/devtools.
  // eslint-disable-next-line no-console
  console.warn("[main] attachConsole fallito (Tauri non disponibile?):", e);
});

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
