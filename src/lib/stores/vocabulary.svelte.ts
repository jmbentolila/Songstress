import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "../window";
import { library } from "./library.svelte";
import type { FieldKey } from "../tagFields";
import type { SuggestionValue } from "../suggest";

/**
 * The tag editors' suggestion vocabulary (owner ask, 2026-09-26).
 *
 * Source of truth is the DB (`tag_vocabulary`, library/vocab.rs — the four tag
 * fields are persisted by migration v6), because reading every file's tags to
 * build a list would cost a full lofty pass per editor open. Two consequences
 * worth knowing:
 *
 *  - The lists are as fresh as the last SCAN. A save writes the file; the
 *    watcher's incremental scan then re-reads it (mtime changed) and the new
 *    value shows up. Until the first Full Rescan after upgrading, the four
 *    tag fields are simply empty — the scan never stored them before v6.
 *  - In browser dev (no Tauri) there is no command to call, so the lists fall
 *    back to what the fake library knows: album titles and artist names. That
 *    keeps the popover exercisable in a browser; the four tag fields stay
 *    quiet, which is honest.
 */

interface Payload {
  artist: SuggestionValue[];
  album_artist: SuggestionValue[];
  album: SuggestionValue[];
  genre: SuggestionValue[];
  composer: SuggestionValue[];
  label: SuggestionValue[];
  grouping: SuggestionValue[];
}

/** The fields that offer completions, and the payload key each one reads. */
const SOURCE: Partial<Record<FieldKey, keyof Payload>> = {
  artist: "artist",
  album: "album",
  albumArtist: "album_artist",
  genre: "genre",
  composer: "composer",
  label: "label",
  grouping: "grouping",
};

/** Refetch threshold. The query is a handful of GROUP BYs over the tracks
 *  table (single-digit ms on a 4.6k-track library), so "fetch per editor open,
 *  at most every few seconds" is cheaper than any invalidation scheme. */
const MAX_AGE_MS = 4000;

export const vocabulary = $state({
  data: null as Payload | null,
  /** epoch ms of the last successful load; 0 = never */
  loadedAt: 0,
  /** Why the last load failed, "" when it did not — the editors show nothing
   *  rather than an error, but the field reports it in dev. */
  error: "",
  loading: false,
});

export function isSuggestable(key: FieldKey): boolean {
  return key in SOURCE;
}

/** Local stand-ins when there is no DB command to ask (browser dev). */
function fallback(key: FieldKey): SuggestionValue[] {
  if (key === "album") {
    return library.albums.map((a) => ({ value: a.title, count: 1 }));
  }
  if (key === "artist" || key === "albumArtist") {
    const counts = new Map<string, number>();
    for (const a of library.albums) {
      const name = library.artistOf(a)?.name ?? "";
      if (name.trim()) counts.set(name, (counts.get(name) ?? 0) + 1);
    }
    return [...counts].map(([value, count]) => ({ value, count }));
  }
  return [];
}

/** The suggestion values for one field — DB vocabulary when there is one,
 *  the fake library's knowledge otherwise. */
export function valuesFor(key: FieldKey): SuggestionValue[] {
  const source = SOURCE[key];
  if (!source) return [];
  return vocabulary.data?.[source] ?? fallback(key);
}

/** Load (or refresh) the vocabulary. Concurrent callers share one request. */
let inFlight: Promise<void> | null = null;

export function loadVocabulary(maxAgeMs = MAX_AGE_MS): Promise<void> {
  if (inFlight) return inFlight;
  if (Date.now() - vocabulary.loadedAt < maxAgeMs) return Promise.resolve();
  if (!isTauri) {
    vocabulary.loadedAt = Date.now();
    return Promise.resolve();
  }
  vocabulary.loading = true;
  inFlight = invoke<Payload>("tag_vocabulary")
    .then((data) => {
      vocabulary.data = data;
      vocabulary.loadedAt = Date.now();
      vocabulary.error = "";
    })
    .catch((e) => {
      vocabulary.error = String(e);
    })
    .finally(() => {
      vocabulary.loading = false;
      inFlight = null;
    });
  return inFlight;
}