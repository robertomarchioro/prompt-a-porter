import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
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
  },
});
