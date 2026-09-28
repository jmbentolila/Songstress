import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./window";

/**
 * Per-track waveform peaks, decoded + cached by the backend (`track_peaks`
 * command) and normalized to amplitudes in [0, 1]. Cached in a Map so a track
 * is fetched once per session; a failure is NOT cached, so it can heal (the
 * same shape as `artColors.ts`).
 */
const cache = new Map<string, Promise<number[]>>();

/** How many tracks the in-memory cache keeps before evicting the oldest. The
 *  backend's SQLite cache is the real store (a re-fetch is ~1ms), so this only
 *  bounds MEMORY: without it the map grows with every track played. */
const CACHE_CAP = 256;

export function trackPeaks(trackId: string): Promise<number[]> {
  const cached = cache.get(trackId);
  if (cached !== undefined) return cached;

  if (!isTauri) return Promise.resolve<number[]>([]);

  const promise = invoke<number[]>("track_peaks", { trackId })
    .then((raw) => (Array.isArray(raw) ? raw.map((v) => v / 255) : []))
    .catch(() => {
      cache.delete(trackId);
      return [] as number[];
    });

  cache.set(trackId, promise);
  // Evict FIFO request order. New inserts land at the end; never evict the id
  // we just inserted in the one-entry/zero-cap corner, merely cap the total.
  let oldest: string | undefined;
  for (const [key] of cache) {
    oldest = key;
    break;
  }
  while (cache.size > CACHE_CAP && oldest !== undefined && oldest !== trackId) {
    cache.delete(oldest);
    oldest = undefined;
    for (const [key] of cache) {
      oldest = key;
      break;
    }
  }
  return promise;
}

/**
 * Warm the cache for tracks likely to play soon, ONE AT A TIME so a burst of
 * decodes never competes with the UI or with playback. Already-cached ids are
 * skipped, and an id already queued is not added twice. A track that the
 * playbar is fetching RIGHT NOW shares its in-flight promise through the cache,
 * so nothing is decoded twice.
 */
const queue: string[] = [];
let draining = false;

export function prefetchPeaks(ids: string[]): void {
  for (const id of ids) {
    if (id && !cache.has(id) && !queue.includes(id)) queue.push(id);
  }
  if (!draining) void drain();
}

async function drain(): Promise<void> {
  draining = true;
  while (queue.length) {
    const id = queue.shift()!;
    if (!cache.has(id)) await trackPeaks(id);
  }
  draining = false;
}