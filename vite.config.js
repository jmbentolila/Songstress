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
