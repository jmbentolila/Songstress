// DEV-ONLY devtools bridge for the Songstress app (agent visual loop).
//
// Adds two loopback endpoints to the Vite dev server (dev mode only):
//   POST /__songstress       app -> us: captured console/IPC/error events,
//                            appended to logs/devtools.log
//   POST /__songstress_cmd   us -> app: a JSON command, relayed to the webview
//                            over the Vite HMR websocket (custom event
//                            "songstress:cmd"), executed by src/lib/devtools.ts
//   GET  /__songstress_log   us -> app: tail of the log file
//
// The CLI wrapper is tools/devctl.mjs. Nothing here runs in production
// builds (configureServer only exists in dev).

import { appendFile, mkdir, readFile } from "node:fs/promises";
import path from "node:path";

const LOG = path.resolve(process.cwd(), "logs/devtools.log");

function readLogTail(n) {
  return readFile(LOG, "utf8")
    .then((txt) => txt.split("\n").slice(-n).join("\n"))
    .catch(() => "(no log yet)");
}

export function songstressDevTools() {
  let wsServer = null;

  const logLine = (line) => {
    const stamped = `[${new Date().toISOString()}] ${line}`;
    // eslint-disable-next-line no-console
    console.log(stamped);
    mkdir(path.dirname(LOG), { recursive: true })
      .then(() => appendFile(LOG, stamped + "\n"))
      .catch(() => {});
  };

  return {
    name: "songstress-devtools",
    configureServer(server) {
      wsServer = server.ws;

      server.middlewares.use((req, res, next) => {
        const url = new URL(req.url ?? "", "http://localhost");
        if (url.pathname === "/__songstress" && req.method === "POST") {
          let body = "";
          req.on("data", (c) => (body += c));
          req.on("end", () => {
            logLine(body.slice(0, 100_000));
            res.writeHead(204).end();
          });
          return;
        }
        if (url.pathname === "/__songstress_cmd" && req.method === "POST") {
          let body = "";
          req.on("data", (c) => (body += c));
          req.on("end", () => {
            try {
              if (!wsServer) throw new Error("ws not ready");
              wsServer.send({ type: "custom", event: "songstress:cmd", data: JSON.parse(body) });
              res.writeHead(200).end("ok");
            } catch (e) {
              res.writeHead(500).end(String(e));
            }
          });
          return;
        }
        if (url.pathname === "/__songstress_log" && req.method === "GET") {
          const n = Math.min(500, Number(url.searchParams.get("n") ?? 200));
          readLogTail(n).then((txt) => res.writeHead(200, { "content-type": "text/plain" }).end(txt));
          return;
        }
        next();
      });
    },
  };
}
