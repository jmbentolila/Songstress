/**
 * Dev-only cover-URL rewrite (WebKitGTK 2.54 regression, found 2026-09-26).
 *
 * WebKitGTK 2.54 stopped letting a page served from an `http://` origin load
 * custom-scheme (`thumb://…`) subresources: the register_uri_scheme_protocol
 * handler is NEVER called (probe-verified: top-level navigation to a custom
 * scheme still fires; an <img> from an http page does not; from a custom-
 * scheme page it does, as long as the scheme is not registered
 * display_isolated). So the dev instance — served by vite over
 * http://localhost:1420 — renders 250 broken covers, while the production
 * RPM (page origin `tauri://localhost`, custom→custom) is unaffected.
 *
 * Fix: in DEV only, rewrite `thumb://…` to `/thumb-http/…`, which the vite
 * middleware in `vite.thumb-dev.js` serves straight from the dev cache dir
 * (same size-fallback rules as the Rust handler). Production builds never
 * see this rewrite — `import.meta.env.DEV` is false and the custom scheme
 * keeps working there.
 */
export function artSrc(url: string): string {
  if (import.meta.env.DEV && url.startsWith("thumb://")) {
    return "/thumb-http/" + url.slice("thumb://".length);
  }
  return url;
}
