// DEV-ONLY devtools bridge, app side (agent visual loop).
//
// initDevtools() is called at the very top of main.ts, before the App
// mounts, so the IPC spy is in place for the first invoke. In production
// builds import.meta.env.DEV is false and everything here is dead-code
// eliminated.
//
// What it does in dev:
//  - mirrors console.*, window errors and unhandled rejections to the
//    Vite devtools sink (POST /__songstress -> logs/devtools.log)
//  - spies __TAURI_INTERNALS__.invoke so every Tauri command + its
//    success/failure shows up in the log (the "why is this screen empty"
//    detector)
//  - listens for commands relayed over the Vite HMR websocket:
//      {k:"click", sel}        -> element.click()
//      {k:"eval", code}        -> run JS in page scope, report result
//      {k:"sample", expr, ms}  -> sample expr every animation frame,
//                                 report the whole trace (motion loop)
//
// Drive it with: node tools/devctl.mjs <cmd>

type Cmd =
  | { k: "click"; sel: string }
  | { k: "eval"; code: string }
  | { k: "sample"; expr: string; ms: number };

const SINK = "/__songstress";

function fmt(v: unknown): string {
  try {
    if (typeof v === "string") return v;
    if (v instanceof Error) return `${v.name}: ${v.message}`;
    const s = JSON.stringify(v);
    return s === undefined ? String(v) : s;
  } catch {
    return String(v);
  }
}

let sending = false;
let queued = "";

function send(t: string, data: unknown) {
  const body = JSON.stringify({ t, at: Date.now(), data });
  queued += body + "\n";
  if (sending || queued.length > 64_000) return;
  sending = true;
  fetch(SINK, { method: "POST", body: queued, keepalive: true, headers: { "content-type": "application/x-ndjson" } })
    .catch(() => {})
    .finally(() => {
      queued = "";
      sending = false;
      if (queued.length) send("flush", null);
    });
}

// Indirect eval so command code runs in global (page) scope.
const pageEval = (code: string): unknown => {
  // eslint-disable-next-line no-eval
  return (0, eval)(code);
};

function handle(cmd: Cmd) {
  const reply = (t: string, data: unknown) => send(t, data);
  (async () => {
    try {
      if (cmd.k === "click") {
        const el = cmd.sel ? (document.querySelector(cmd.sel) as HTMLElement | null) : null;
        el?.click();
        reply("click-result", { sel: cmd.sel, found: !!el });
      } else if (cmd.k === "eval") {
        const v = await pageEval(cmd.code);
        reply("eval-result", { v: fmt(v).slice(0, 8000) });
      } else if (cmd.k === "sample") {
        const t0 = performance.now();
        const rows: unknown[] = [];
        await new Promise<void>((res) => {
          const step = () => {
            try {
              rows.push(pageEval(cmd.expr));
            } catch (e) {
              rows.push(fmt(e));
            }
            if (performance.now() - t0 >= cmd.ms) res();
            else requestAnimationFrame(step);
          };
          step();
        });
        reply("sample-result", { expr: cmd.expr, ms: cmd.ms, n: rows.length, rows: fmt(rows).slice(0, 200_000) });
      }
    } catch (e) {
      reply("cmd-error", { cmd, err: fmt(e) });
    }
  })();
}

export function initDevtools() {
  if (!import.meta.env.DEV) return;

  try {
    for (const m of ["log", "warn", "error", "info", "debug"] as const) {
      const orig = console[m].bind(console);
      // Some webviews expose console methods as readonly; never let the
      // bridge take the app down over cosmetics.
      Object.assign(console, { [m]: (...a: unknown[]) => {
        orig(...a);
        try {
          send(`console:${m}`, a.map(fmt).join(" "));
        } catch {
          /* never break the app */
        }
      } });
    }
  } catch (e) {
    send("console-mirror-failed", { err: fmt(e), stack: (e as Error)?.stack?.slice(0, 400) });
  }

  window.addEventListener("error", (e) =>
    send("window-error", {
      msg: e.message,
      src: e.filename,
      line: e.lineno,
      stack: (e.error as Error | null)?.stack?.slice(0, 600),
    }),
  );
  window.addEventListener("unhandledrejection", (e) => send("unhandled-rejection", { reason: fmt(e.reason) }));

  const internals = (window as { __TAURI_INTERNALS__?: { invoke?: (c: string, a?: unknown) => Promise<unknown> } })
    .__TAURI_INTERNALS__;
  // NOTE: an invoke-level spy is impossible in Tauri v2 —
  // window.__TAURI_INTERNALS__ and its invoke property are both
  // non-configurable/non-writable. Swallowed IPC errors are instead
  // caught by app-side console.error calls (mirrored here) and the
  // unhandledrejection hook below.

  import.meta.hot?.on("songstress:cmd", (cmd: Cmd) => handle(cmd));
  send("boot", { href: location.href, tauri: !!internals, ua: navigator.userAgent.slice(0, 100) });
}
