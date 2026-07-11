import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

export default defineConfig({
  // #462: plugin Svelte necessario per compilare i componenti .svelte
  // importati dai test di regressione (es. DiffViewer.test.ts).
  plugins: [svelte()],
  test: {
    include: ["src/**/*.test.ts"],
    coverage: {
      provider: "v8",
      include: ["src/lib/**/*.ts"],
      exclude: [
        "src/lib/**/*.test.ts",
        "src/lib/**/*.d.ts",
        "src/lib/**/index.ts",
        // Runes-based stores (richiedono runtime Svelte)
        "src/lib/**/*.svelte.ts",
        // Wrapper di Tauri invoke / estensioni CodeMirror runtime
        "src/lib/sync.ts",
        "src/lib/codemirror/import-autocomplete.ts",
        "src/lib/codemirror/placeholder-highlight.ts",
      ],
      reporter: ["text", "html", "lcov", "json-summary"],
      thresholds: {
        lines: 70,
        functions: 70,
        statements: 70,
        branches: 70,
      },
    },
  },
  resolve: {
    alias: {
      $lib: path.resolve(__dirname, "src/lib"),
    },
    // #462: forza la condizione "browser" così @sveltejs/vite-plugin-svelte
    // compila i componenti in client-mode invece di SSR-mode sotto Vitest —
    // altrimenti `mount()` di @testing-library/svelte fallisce con
    // "lifecycle_function_unavailable" (mount non è disponibile server-side).
    conditions: ["browser"],
  },
});
