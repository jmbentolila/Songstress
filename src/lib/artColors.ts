import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./window";

export type RGB = [number, number, number];

const cache = new Map<string, Promise<[RGB, RGB] | null>>();

export function extractArtColors(src: string): Promise<[RGB, RGB] | null> {
  const cached = cache.get(src);
  if (cached !== undefined) return cached;

  if (!isTauri) {
    const none = Promise.resolve<[RGB, RGB] | null>(null);
    cache.set(src, none);
    return none;
  }

  const file = src.split("/").pop() ?? "";
  const promise = invoke<number[]>("album_colors", { file })
    .then((raw): [RGB, RGB] | null => {
      if (!Array.isArray(raw) || raw.length !== 6) return null;
      return [
        [raw[0], raw[1], raw[2]],
        [raw[3], raw[4], raw[5]],
      ];
    })
    .catch(() => null);

  // Cache successes only; failures stay uncached so they can heal later.
  void promise.then((v) => {
    if (v === null) cache.delete(src);
  });
  cache.set(src, promise);
  return promise;
}
