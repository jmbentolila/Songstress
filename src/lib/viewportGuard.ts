import { invoke } from "@tauri-apps/api/core";
import { appWindow } from "./window";

const FIRST_CHECK_MS = 4000;
const LAST_CHECK_MS = 60000;
const CHECK_INTERVAL_MS = 1000;
const TOLERANCE_PX = 8;
const COOLDOWN_CHECKS = 4;

export function startViewportGuard() {
  if (!appWindow) return;
  let elapsed = FIRST_CHECK_MS;
  let cooldown = 0;
  const timer = setInterval(() => {
    elapsed += CHECK_INTERVAL_MS;
    if (elapsed > LAST_CHECK_MS) {
      clearInterval(timer);
      return;
    }
    if (cooldown > 0) {
      cooldown -= 1;
      return;
    }
    void (async () => {
      try {
        const size = await appWindow!.innerSize();
        const dpr = window.devicePixelRatio || 1;
        const wantW = Math.round(window.innerWidth * dpr);
        const wantH = Math.round(window.innerHeight * dpr);
        if (
          Math.abs(size.width - wantW) > TOLERANCE_PX * dpr ||
          Math.abs(size.height - wantH) > TOLERANCE_PX * dpr
        ) {
          await invoke("fix_viewport");
          cooldown = COOLDOWN_CHECKS;
        }
      } catch {
        // window gone or IPC hiccup — not fatal, retry next tick
      }
    })();
  }, CHECK_INTERVAL_MS);
}
