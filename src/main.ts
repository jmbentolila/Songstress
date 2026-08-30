import { mount } from "svelte";
import "./app.css";
import { initDevtools } from "./lib/devtools";
import App from "./App.svelte";

// DEV-only devtools bridge (no-op in prod): console/IPC/error sink +
// command channel. Must run before App mounts so the invoke spy is in
// place for the first Tauri call. See src/lib/devtools.ts + tools/devctl.mjs.
initDevtools();

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
