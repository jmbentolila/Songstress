#!/usr/bin/env node
// WebKitGTK remote-inspector probe (dev only).
//
// The dev unit sets WEBKIT_INSPECTOR_SERVER=127.0.0.1:9301 (drop-in in
// ~/.config/systemd/user/songstress-dev.service.d/inspector.conf), which
// makes the app's webview expose its inspector on loopback. This script
// speaks the inspector's CDP-like protocol over a plain WebSocket so the
// agent (or a human) can read the webview's console and evaluate JS in
// the app's context without touching the desktop:
//
//   node tools/inspect.mjs eval "<expression>"        # evaluate, print, exit
//   node tools/inspect.mjs console [seconds]          # stream console messages
//   node tools/inspect.mjs run "<expression>" [secs]  # both
//
// Expressions run in the app page's scope; Tauri IPC is reachable via
// window.__TAURI_INTERNALS__.invoke(...). Examples:
//   node tools/inspect.mjs eval "document.title"
//   node tools/inspect.mjs eval "__TAURI_INTERNALS__.invoke('get_library').then(d=>d.albums.length)"
//   node tools/inspect.mjs run "document.querySelector('.tile')?.click()" 5

const PORT = process.env.INSPECTOR_PORT ?? "9301";
const mode = process.argv[2] ?? "console";
const arg = process.argv[3];
const secs = Number(process.argv[4] ?? (mode === "console" ? 8 : 3));

const ws = new WebSocket(`ws://127.0.0.1:${PORT}`);
let seq = 0;
const pending = new Map();
const done = (code = 0) => {
  try {
    ws.close();
  } catch {}
  setTimeout(() => process.exit(code), 50).unref();
};

const reply = (id, value, exception) => {
  const p = pending.get(id);
  if (!p) return;
  pending.delete(id);
  if (exception) console.log(`[exception] ${exception.exceptionDetails?.text ?? exception}`);
  const v = value?.result?.value ?? value?.result;
  console.log(typeof v === "string" ? v : JSON.stringify(v, null, 2));
  p();
};

function send(method, params = {}) {
  const id = ++seq;
  return new Promise((res) => {
    pending.set(id, res);
    ws.send(JSON.stringify({ id, method, params }));
  });
}

const printConsole = (m) => {
  const args = (m.params?.args ?? []).map((a) => a.value ?? a.description ?? a.type).join(" ");
  const loc = m.params?.location ? ` (line ${m.params.location.lineNumber})` : "";
  console.log(`[console:${m.params?.type ?? "log"}] ${args}${loc}`);
};

ws.onmessage = (ev) => {
  const txt = String(ev.data);
  if (txt.startsWith("::")) return; // `:: connected` handshake line
  let m;
  try {
    m = JSON.parse(txt);
  } catch {
    return;
  }
  if (m.id && pending.has(m.id)) reply(m.id, m.result, m.result?.exceptionDetails ? m.result : null);
  else if (m.method === "Runtime.consoleAPICalled") printConsole(m);
  else if (m.method === "Runtime.exceptionThrown")
    console.log(`[uncaught] ${m.params?.exception?.description ?? m.params?.exceptionDetails?.text ?? "?"}`);
};

ws.onerror = (e) => {
  console.error(`inspector websocket error: ${e.message ?? "?"}`);
  done(2);
};

ws.onopen = async () => {
  await send("Runtime.enable");
  if (mode === "eval") {
    await send("Runtime.evaluate", {
      expression: arg,
      awaitPromise: true,
      returnByValue: true,
      timeout: 15000,
    });
    done();
  } else if (mode === "run") {
    await send("Runtime.evaluate", {
      expression: arg,
      awaitPromise: true,
      returnByValue: true,
      timeout: 15000,
    });
    setTimeout(done, secs * 1000); // keep listening for console output
  } else {
    setTimeout(done, secs * 1000); // console mode: just listen
  }
};

setTimeout(() => {
  console.error("timeout waiting for inspector (is the dev unit running with the inspector drop-in?)");
  process.exit(3);
}, 8000).unref();
