import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Album, Artist, Track } from "../types";
import { albums as fakeAlbums, artistOf as fakeArtistOf, artists as fakeArtists, tracksOf as fakeTracksOf } from "../fakeLibrary";
import { sortKey } from "../sort";
import { MIN_SKELETON_MS, ENTER_MS } from "../loadingState";
import { isTauri } from "../window";
import { pushSetting, ui } from "./ui.svelte";
import { announcer } from "./announcer.svelte";

/**
 * Live mode is the default inside Tauri; `VITE_LIB=fake` opts out (and the
 * plain browser always gets fake data — there's no backend to ask).
 */
export const LIVE_LIBRARY = isTauri && import.meta.env.VITE_LIB !== "fake";

/** Albums ordered by year ascending everywhere; unknown years sink, titles
 * break ties (article-stripping via the shared sort key). */
function byYear(albums: Album[]): Album[] {
  return [...albums].sort((a, b) => {
    if (a.year === null && b.year !== null) return 1;
    if (b.year === null && a.year !== null) return -1;
    if (a.year !== b.year) return (a.year ?? 0) - (b.year ?? 0);
    return sortKey(a.title).localeCompare(sortKey(b.title));
  });
}

interface LibraryDump {
  artists: Artist[];
  albums: Album[];
  tracks: Track[];
}

class LibraryStore {
  live = LIVE_LIBRARY;
  /** False while a live library hasn't loaded yet (first launch = empty). */
  ready = $state(!LIVE_LIBRARY);
  scanning = $state(false);
  /** When the store began — the boot skeleton's visible clock (see
   * `MIN_SKELETON_MS`). Constructed at module import, i.e. before first paint,
   * so the floor errs a hair generous, never short. */
  #bootAt = Date.now();
  /** DEV-only: drive the loading state without a scan in flight (`__skel` in
   * main.ts). Read by `libraryLoading` — and it short-circuits the "never over
   * content" rule on purpose: forcing it ON a populated library is how you see
   * what the placeholder looks like at the real column count and theme. */
  devLoading = $state(false);
  /** Why the LAST scan attempt failed, "" when it didn't. The empty state's
   * failure branch is the only thing a scan error has ever had to say in this
   * app — `scan_library`'s Err reached the webview console and nowhere else, and
   * the console has no window (AGENTS.md: "scan errors are near-invisible"). */
  scanError = $state("");
  artists = $state<Artist[]>(LIVE_LIBRARY ? [] : fakeArtists);
  albums = $state<Album[]>(LIVE_LIBRARY ? [] : byYear(fakeAlbums));
  /** Denominator for the Library pane's status footer. */
  trackCount = $state(0);
  #byAlbum = new Map<string, Track[]>();

  constructor() {
    if (LIVE_LIBRARY) {
      void this.load();
      // No grace timer any more: the boot skeleton is on from the first frame
      // on EVERY launch (owner ruling 2026-09-06 — hiding a warm launch's dump
      // made content appear by pop), and `load()` holds the first dump out for
      // `MIN_SKELETON_MS` so the placeholder reads as intentional instead of
      // a 50 ms flicker.
      void listen("scan-finished", (e) => {
        // This event is the ONLY thing that clears `scanning`, so the backend
        // emits it on every path now (lib.rs::scan_inner) — including the failure
        // and panic paths that used to return before the emit. When that was true
        // a failed FIRST scan left `scanning` set forever: an eternal shimmer over
        // an empty grid, with the reason in a console nobody can open.
        this.scanning = false;
        const err = (e.payload as { error?: string | null }).error ?? "";
        this.scanError = err;
        // The same sentence the chrome shows, heard (WCAG 4.1.3): a scan
        // that failed while you watched something else must reach you.
        announcer.say(err ? `Scan failed: ${err}` : "Library updated");
        if (err) {
          // Nothing was written — the run stops before grouping and the upserts —
          // so there is no new dump to read, and the footer must not stamp
          // "scanned 14:02" for a scan that did not happen.
          console.error("[scan] " + err);
          return;
        }
        // "scanned 14:02" in the Library footer — persisted, so the answer
        // survives a restart instead of being session-fresh every launch.
        ui.lastScan = Date.now();
        pushSetting("lastScan", ui.lastScan);
        void this.load(true);
      });
      void listen("scan-progress", () => {
        // The only signal that a scan the FRONTEND never asked for is running —
        // a watcher-triggered first scan sets this and nothing else.
        this.scanning = true;
      });
    }
  }

  /** Armed for one entrance: the frames right after a placeholder gave way to
   * content. The grid and the artist list both read it (`class:enter`) so they
   * arrive on the same beat — one dump, one cascade — and it is set here rather
   * than from a component `$effect` because an effect runs after the DOM update:
   * the class would land a frame late, each row would paint at rest and then jump
   * to the start of its animation, and the whole thing would blink. */
  entering = $state(false);
  #enterTimer: ReturnType<typeof setTimeout> | undefined;

  async load(fromScan = false) {
    try {
      const dump = await invoke<LibraryDump>("get_library");
      const firstDump = !this.ready;
      if (firstDump) {
        // The floor, not a delay: the skeleton has been on screen since the
        // first frame; releasing it into content 50 ms later is a flicker,
        // not a hand-over. Slow (cold) dumps simply arrive past the floor.
        const remain = MIN_SKELETON_MS - (Date.now() - this.#bootAt);
        if (remain > 0) await new Promise((r) => setTimeout(r, remain));
      }
      // The entrance belongs to the WAIT, not to the data: it fires when a
      // placeholder is what is being replaced. Boot always showed one (and
      // held the floor), so every launch arrives on the cascade now — warm
      // launches included (owner ruling 2026-09-06); a mid-session refresh
      // over visible content still swaps silently.
      const filledFromPlaceholder =
        dump.albums.length > 0 &&
        (firstDump || (this.albums.length === 0 && (fromScan || this.scanning)));
      this.artists = dump.artists;
      this.albums = byYear(dump.albums);
      this.trackCount = dump.tracks.length;
      this.#byAlbum = new Map<string, Track[]>();
      for (const t of dump.tracks) {
        const list = this.#byAlbum.get(t.albumId);
        if (list) list.push(t);
        else this.#byAlbum.set(t.albumId, [t]);
      }
      this.ready = true;
      if (filledFromPlaceholder) this.#enterNow();
    } catch {
      // Backend hiccup — keep whatever we have; next scan-finished retries.
    }
  }

  #enterNow() {
    this.entering = true;
    clearTimeout(this.#enterTimer);
    // And it must come back OFF. Left on, every later filter change would push its
    // freshly-mounted rows through the entrance — an artist switch or a search
    // keystroke is precisely the frequency tier that disqualifies motion.
    this.#enterTimer = setTimeout(() => (this.entering = false), ENTER_MS);
  }

  tracksOf(albumId: string): Track[] {
    return this.live ? (this.#byAlbum.get(albumId) ?? []) : fakeTracksOf(albumId);
  }

  /** Flag a track as missing without waiting for a rescan (the file just
   * failed to load) so the alert icon + locate flow appear immediately. */
  markMissing(trackId: string) {
    for (const list of this.#byAlbum.values()) {
      const t = list.find((x) => x.id === trackId);
      if (t) {
        t.missing = true;
        return;
      }
    }
  }

  artistOf(album: Album): Artist | undefined {
    return this.live
      ? this.artists.find((a) => a.id === album.artistId)
      : fakeArtistOf(album);
  }
}

export const library = new LibraryStore();
