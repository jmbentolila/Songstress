#!/usr/bin/env node
// Songstress devtools CLI — drive + observe the dev app without touching
// the desktop. Requires the dev server (systemctl --user status
// songstress-dev) and the devtools bridge (vite.songstress.devtools.js).
//
//   node tools/devctl.mjs click "<selector>"            # click an element
//   node tools/devctl.mjs eval "<js expression>"        # eval in page scope
//   node tools/devctl.mjs sample "<js expression>" <ms> # rAF-sample a trace
//   node tools/devctl.mjs tail [n]                      # last n log lines
//   node tools/devctl.mjs clear                         # truncate the log
//
// The log is also written to logs/devtools.log — `tail` just reads it via
// the dev server. Examples:
//   node tools/devctl.mjs click ".tile"
//   node tools/devctl.mjs eval "document.querySelector('.row')?.getBoundingClientRect().y"
//   node tools/devctl.mjs sample "document.querySelector('.row')?.getBoundingClientRect().y" 900

const BASE = process.env.SONGSTRESS_DEV_URL ?? "http://[::1]:1420";
const [cmd, a, b] = process.argv.slice(2);

const post = async (path, body) => {
  const res = await fetch(BASE + path, {
    method: "POST",
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${path} -> HTTP ${res.status}: ${await res.text()}`);
  return res.text();
};

switch (cmd) {
  case "click":
    await post("/__songstress_cmd", { k: "click", sel: a });
    console.log(`sent click "${a}" — check: node tools/devctl.mjs tail 20`);
    break;
  case "eval":
    await post("/__songstress_cmd", { k: "eval", code: a });
    console.log("sent eval — check: node tools/devctl.mjs tail 20");
    break;
  case "sample": {
    const ms = Number(b ?? 600);
    await post("/__songstress_cmd", { k: "sample", expr: a, ms });
    console.log(`sampling ${ms}ms — check after ~${(ms / 1000 + 0.5).toFixed(1)}s: node tools/devctl.mjs tail 30`);
    break;
  }
  case "tail": {
    const n = Number(a ?? 200);
    const res = await fetch(`${BASE}/__songstress_log?n=${n}`);
    process.stdout.write(await res.text());
    break;
  }
  case "clear":
    await post("/__songstress_cmd", { k: "eval", code: "1" }); // liveness ping (harmless)
    await fetch(`${BASE}/__songstress_log`).then(() => {});
    console.log("use: rm logs/devtools.log");
    break;
  default:
    console.log("usage: devctl.mjs click <sel> | eval <js> | sample <js> <ms> | tail [n]");
    process.exit(1);
}
