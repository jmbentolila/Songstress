import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { songstressDevTools } from "./vite.songstress.devtools.js";

export default defineConfig({
  // songstressDevTools: DEV-ONLY bridge for the agent visual loop
  // (sink at POST /__songstress, commands at /__songstress_cmd, log at
  // logs/devtools.log). No-op outside dev. See tools/devctl.mjs.
  plugins: [svelte(), songstressDevTools()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Cold-start race (seen 2026-08-31): the webview asks for
    // `X.svelte?svelte&type=style&lang.css` in the same burst that first
    // requests `X.svelte`. If the style subrequest lands before the plugin has
    // compiled its parent, the plugin has no metadata for the id, Vite falls
    // through to the static-file handler (which ignores the query) and the
    // "CSS" comes back as the component's RAW SOURCE. The style tag then never
    // injects and the app renders layout-less — shell components (App,
    // AlbumGrid, PlayBar) lose their layout while later components are fine,
    // which looks like "only the sidebar broke". Reloading does NOT fix it
    // (the id stays uncached); editing/touching the file does.
    // Pre-transforming the entry graph at startup removes the window.
    warmup: {
      clientFiles: ["./src/main.ts"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    target: "chrome105",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
