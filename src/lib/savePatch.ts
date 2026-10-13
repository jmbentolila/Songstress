import type { Album, Artist, Track } from "./types";

/** One relinked track: the row the dump holds under `oldId` continues under
 *  `newId` — same tags, same album, only the path-derived id moved. */
export interface SavePatchMove {
  oldId: string;
  newId: string;
}

/**
 * Everything one save's dump must change, carried on that save's own
 * `scan-finished` so the ~450ms full `get_library` round trip is skipped.
 * `moved` renames rows in place (staged rows become ordinary library rows);
 * `removed` drops staged rows the library already held (duplicates) or that
 * vanished. Album/artist rows never change on a save — their ids are
 * tag-derived — so albums left empty along the way are pruned, exactly as a
 * fresh dump (which only ever contains non-empty albums) would show them.
 */
export interface SavePatch {
  moved: SavePatchMove[];
  removed: string[];
}

/**
 * Pure core of the save-adjacent refresh: patch a dump in place-order.
 * Kept pure (and covered in savePatch.test.ts) so the store stays a thin
 * assignment shell over it.
 *
 * Idempotent: unknown ids apply as skips and count nothing, so applying the
 * same patch twice — or racing a full load on either side of it — converges
 * instead of duplicating. `applied` is the number of rows actually changed;
 * the store falls back to a full load when a non-empty patch applies
 * nothing, because that means the dump moved under us.
 */
export function applySavePatchToDump(
  artists: Artist[],
  albums: Album[],
  tracks: Track[],
  patch: SavePatch,
): { artists: Artist[]; albums: Album[]; tracks: Track[]; applied: number } {
  const removed = new Set(patch.removed);
  const renamed = new Map(patch.moved.map((m) => [m.oldId, m.newId]));
  const touchedAlbums = new Set<string>();
  let applied = 0;
  const nextTracks: Track[] = [];
  for (const t of tracks) {
    if (removed.has(t.id)) {
      touchedAlbums.add(t.albumId);
      applied++;
      continue;
    }
    const newId = renamed.get(t.id);
    if (newId !== undefined) {
      // Same row, new identity; the backend already cleared staged on it.
      touchedAlbums.add(t.albumId);
      applied++;
      nextTracks.push({ ...t, id: newId, staged: false });
    } else {
      nextTracks.push(t);
    }
  }
  // Albums survive iff they still hold tracks (order preserved — drops only).
  // Touched ones recompute the derived `staged` flag exactly the way a fresh
  // dump does; untouched ones cannot have changed.
  const live = new Map<string, Track[]>();
  for (const t of nextTracks) {
    const list = live.get(t.albumId);
    if (list) list.push(t);
    else live.set(t.albumId, [t]);
  }
  const nextAlbums = albums
    .filter((a) => live.has(a.id))
    .map((a) =>
      touchedAlbums.has(a.id)
        ? { ...a, staged: live.get(a.id)!.some((t) => t.staged) }
        : a,
    );
  const liveArtistIds = new Set(nextAlbums.map((a) => a.artistId));
  const nextArtists = artists.filter((a) => liveArtistIds.has(a.id));
  return { artists: nextArtists, albums: nextAlbums, tracks: nextTracks, applied };
}
