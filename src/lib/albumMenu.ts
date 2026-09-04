import { invoke } from "@tauri-apps/api/core";
import { library, LIVE_LIBRARY } from "./stores/library.svelte";
import { ui } from "./stores/ui.svelte";
import { queueTracks, playTrack } from "./stores/playback.svelte";
import {
  openContextMenu,
  SEP,
  type MenuItem,
} from "./stores/contextMenu.svelte";
import {
  removeMissingTracks,
} from "./stores/scanner.svelte";
import { openImportManager } from "./stores/imports.svelte";

/** The album items a right-click offers — ONE builder for the two doors:
 *  a grid tile and the expanded panel's header (they used to be separate
 *  copies of this list; a change to one silently missed the other). The
 *  staged/missing rows appear only when there is something to act on, so a
 *  clean album offers just Edit + Play. */
export function albumMenuItems(albumId: string): MenuItem[] {
  const items: MenuItem[] = [
    {
      label: "Edit album tags…",
      action: () => {
        ui.tagEditor = { open: true, albumId, trackId: null };
      },
    },
    SEP,
    // The pair the tracks already are: play NOW (first file that exists —
    // a missing opener shouldn't make the row a dead click) vs queue NEXT
    // (the whole album, missing files skipped, one door as in the track
    // menu). Used to be one "Play album next" row; "play it" was the
    // thing users actually wanted and had to expand the album for.
    {
      label: "Play album",
      action: () => {
        const i = library.tracksOf(albumId).findIndex((t) => !t.missing);
        if (i >= 0) void playTrack(albumId, i);
      },
    },
    {
      label: "Add album to queue",
      action: () =>
        void queueTracks(
          library.tracksOf(albumId).filter((t) => !t.missing).map((t) => t.id),
          true,
        ),
    },
    SEP,
    // Rust resolves the folder from the DB (majority of the album's
    // files — multi-disc safe); the webview never sees the path. Staged
    // albums work too: "wherever your files are right now" is still
    // their container until Save moves them. Fake-library dev mode has
    // no DB rows, so the row only exists where it can actually act.
    ...(LIVE_LIBRARY
      ? [
          {
            label: "Open containing folder",
            action: () =>
              void invoke("reveal_container", { albumId }).catch((e) =>
                console.error(e),
              ),
          },
        ]
      : []),
  ];
  const album = library.albums.find((a) => a.id === albumId);
  const hasMissing = library.tracksOf(albumId).some((t) => t.missing);
  if (album?.staged || hasMissing) items.push(SEP);
  if (album?.staged) {
    // Not Save/Discard: those were a decision made blind — the destination
    // is what this decision is about, and only the modal states it per
    // album (the sidebar door took exactly this position long ago). Opens
    // the modal AT this album: expand, scroll, flash.
    items.push({
      label: "Manage imported music…",
      action: () => void openImportManager(albumId),
    });
  }
  if (hasMissing) {
    items.push({
      label: "Remove missing tracks",
      action: () => void removeMissingTracks(albumId),
    });
  }
  return items;
}

/** Open the album menu at the pointer. Callers pass the raw MouseEvent; the
 *  default menu is suppressed here so this is the ONLY right-click door for
 *  an album (the browser/WebKit one would otherwise sit behind ours). */
export function openAlbumMenu(e: MouseEvent, albumId: string) {
  e.preventDefault();
  openContextMenu(e.clientX, e.clientY, albumMenuItems(albumId));
}
