import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Album, Artist, Track } from "../types";
import { albums as fakeAlbums, artistOf as fakeArtistOf, artists as fakeArtists, tracksOf as fakeTracksOf } from "../fakeLibrary";
import { sortKey } from "../sort";
import { isTauri } from "../window";
import { pushSetting, ui } from "./ui.svelte";

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
  artists = $state<Artist[]>(LIVE_LIBRARY ? [] : fakeArtists);
  albums = $state<Album[]>(LIVE_LIBRARY ? [] : byYear(fakeAlbums));
  /** Denominator for the Library pane's status footer. */
  trackCount = $state(0);
  #byAlbum = new Map<string, Track[]>();

  constructor() {
    if (LIVE_LIBRARY) {
      void this.load();
      void listen("scan-finished", () => {
        this.scanning = false;
        // "scanned 14:02" in the Library footer — persisted, so the answer
        // survives a restart instead of being session-fresh every launch.
        ui.lastScan = Date.now();
        pushSetting("lastScan", ui.lastScan);
        void this.load();
      });
      void listen("scan-progress", (e) => {
        this.scanning = true;
        const p = e.payload as { done: number; total: number };
        this.scanDone = p.done;
        this.scanTotal = p.total;
      });
    }
  }

  scanDone = $state(0);
  scanTotal = $state(0);

  async load() {
    try {
      const dump = await invoke<LibraryDump>("get_library");
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
    } catch {
      // Backend hiccup — keep whatever we have; next scan-finished retries.
    }
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
