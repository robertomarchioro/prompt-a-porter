import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    host: host || false,
    port: 5173,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 5174 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: process.env.NODE_ENV === "development",
    // F11 PR-C: codemirror chunk legittimamente > 500 kB raw (185 kB
    // gzip, scomponibile solo con feature-flag dev/markdown). Warning
    // disattivato per evitare falso positivo CI.
    chunkSizeWarningLimit: 600,
    // F11 PR-C: manualChunks per cache vendor cross-deploy.
    // Splitta i dep più grossi in bundle separati così che app code
    // updates non invalidano la cache del browser per codemirror/etc.
    rollupOptions: {
      output: {
        manualChunks: {
          codemirror: [
            "codemirror",
            "@codemirror/state",
            "@codemirror/view",
            "@codemirror/language",
            "@codemirror/lang-markdown",
            "@codemirror/autocomplete",
            "@codemirror/commands",
            "@codemirror/search",
          ],
          diff: ["diff2html"],
          icons: ["lucide-svelte"],
        },
      },
    },
  },
});
