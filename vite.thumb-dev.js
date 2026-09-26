// Dev-only thumb bridge (see src/lib/artSrc.ts for the why).
//
// WebKitGTK 2.54 blocks custom-scheme subresources from http pages, so in
// dev the frontend rewrites `thumb://<id>/<file>.webp` to
// `/thumb-http/<id>/<file>.webp`. This middleware serves it from the SAME
// cache dir the Rust side writes, mirroring the `thumb://` handler in
// src-tauri/src/lib.rs: hashed cache-busters (`512-a1b2c3d4.webp`) carry the
// size before the dash; album covers keep the lenient 512→256→96 fallback
// chain; `art-` previews are served strictly (no fallback).
//
// configureServer only exists on the dev server, so a production build can
// never carry this path.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";

// The dev profile's identifier (tauri.dev.conf.json) — app_cache_dir() on
// Linux is ~/.cache/<identifier>.
const CACHE_ROOT = path.join(
  os.homedir(),
  ".cache",
  "com.yossi.songstress.dev",
  "thumbs",
);

export function songstressThumbDev() {
  return {
    name: "songstress-thumb-dev",
    configureServer(server) {
      server.middlewares.use("/thumb-http", (req, res) => {
        const rel = decodeURIComponent((req.url || "").split("?")[0]).replace(/^\/+/, "");
        const [albumId, file] = rel.split("/");
        if (!albumId || !file || albumId.includes("..") || file.includes("..")) {
          res.statusCode = 400;
          res.end();
          return;
        }
        const size = file.replace(/\.webp$/, "").split("-")[0];
        const sizes = [size, ...(albumId.startsWith("art-") ? [] : ["512", "256", "96"])];
        for (const s of sizes) {
          if (!/^\d+$/.test(s)) break; // same reject as the Rust parse::<u32>
          const p = path.join(CACHE_ROOT, albumId, `${s}.webp`);
          try {
            const bytes = fs.readFileSync(p);
            res.setHeader("Content-Type", "image/webp");
            res.statusCode = 200;
            res.end(bytes);
            return;
          } catch {
            /* next candidate */
          }
        }
        res.statusCode = 404;
        res.end();
      });
    },
  };
}
